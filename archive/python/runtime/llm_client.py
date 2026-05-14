"""LangChain model setup, prompt loading, response parsing, and retry logic."""

from __future__ import annotations

import asyncio
import json
import logging
import os
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[3]
MAX_RETRIES = 3
RETRY_BASE_DELAY = 2.0

log = logging.getLogger("generate")


def create_llm(model_string: str):
    """
    Build a LangChain chat model from a '<provider>:<model-id>' string.

    Supported providers: anthropic, openai, ollama.
    """
    if ":" not in model_string:
        raise ValueError(
            f"Invalid model format '{model_string}'. "
            "Expected '<provider>:<model-id>', "
            "e.g. 'openai:gpt-5.4-mini', 'anthropic:claude-haiku-4-5-20251001', "
            "or 'ollama:translategemma:12b'."
        )

    provider, model_id = model_string.split(":", 1)

    if provider == "anthropic":
        _require_env("ANTHROPIC_API_KEY", provider)
        from langchain_anthropic import ChatAnthropic

        return ChatAnthropic(model=model_id, max_tokens=8192)

    if provider == "openai":
        _require_env("OPENAI_API_KEY", provider)
        from langchain_openai import ChatOpenAI

        return ChatOpenAI(model=model_id, max_tokens=8192)

    if provider == "ollama":
        from langchain_openai import ChatOpenAI

        return ChatOpenAI(
            model=model_id,
            base_url=_ollama_base_url(),
            api_key=os.environ.get("OLLAMA_API_KEY", "ollama"),
            max_tokens=8192,
        )

    raise ValueError(f"Unknown provider '{provider}'. Supported providers: anthropic, openai, ollama.")


def _ollama_base_url() -> str:
    base_url = os.environ.get("OLLAMA_BASE_URL", "http://localhost:11434").rstrip("/")
    if base_url.endswith("/v1"):
        return base_url
    return f"{base_url}/v1"


def _require_env(key: str, provider: str) -> None:
    if not os.environ.get(key):
        raise EnvironmentError(
            f"{key} is required for {provider} models. "
            f"Set it with: export {key}=<your-api-key>"
        )


def load_prompt(pipeline_type: str) -> str:
    """Load the generation prompt file for the given pipeline type."""
    path = PROJECT_ROOT / f"generation_prompt_{pipeline_type}.txt"
    if not path.exists():
        raise FileNotFoundError(f"Prompt file not found: {path}")
    return path.read_text(encoding="utf-8")


def build_messages(llm, system_prompt: str, source_content: str) -> list:
    """
    Build the message list for the LLM call.

    Anthropic messages attach cache_control to the system prompt so repeated
    batch calls can reuse the large prompt.
    """
    from langchain_core.messages import HumanMessage, SystemMessage

    if "anthropic" in type(llm).__module__:
        system_content = [
            {
                "type": "text",
                "text": system_prompt,
                "cache_control": {"type": "ephemeral"},
            }
        ]
        return [
            SystemMessage(content=system_content),
            HumanMessage(content=source_content),
        ]

    return [
        SystemMessage(content=system_prompt),
        HumanMessage(content=source_content),
    ]


def extract_json(text: str) -> dict:
    """
    Parse JSON from an LLM response, stripping markdown fences when present.
    """
    text = text.strip()

    if text.startswith("```"):
        lines = text.splitlines()
        inner = lines[1:]
        if inner and inner[-1].strip() == "```":
            inner = inner[:-1]
        text = "\n".join(inner).strip()

    return json.loads(text)


def extract_usage(response) -> dict:
    """Extract token usage counts from a LangChain AI message."""
    result = {
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
    }

    meta = getattr(response, "usage_metadata", None)
    if not meta:
        return result

    if isinstance(meta, dict):
        result["input_tokens"] = meta.get("input_tokens", 0)
        result["output_tokens"] = meta.get("output_tokens", 0)
        details = meta.get("input_token_details", {})
    else:
        result["input_tokens"] = getattr(meta, "input_tokens", 0)
        result["output_tokens"] = getattr(meta, "output_tokens", 0)
        details = getattr(meta, "input_token_details", {}) or {}

    if isinstance(details, dict):
        result["cache_read_tokens"] = details.get("cache_read", 0)
        result["cache_write_tokens"] = details.get("cache_creation", 0)

    return result


async def call_llm(llm, system_prompt: str, source_content: str) -> tuple[dict, dict]:
    """
    Make a single LLM call and return (parsed_json, usage_dict).
    """
    messages = build_messages(llm, system_prompt, source_content)
    response = await llm.ainvoke(messages)
    return extract_json(response.content), extract_usage(response)


async def call_llm_with_retry(
    llm,
    system_prompt: str,
    source_content: str,
    label: str,
) -> tuple[dict, dict]:
    """
    Retry an LLM call with exponential backoff.
    """
    last_error: Exception | None = None

    for attempt in range(1, MAX_RETRIES + 1):
        try:
            return await call_llm(llm, system_prompt, source_content)

        except json.JSONDecodeError as exc:
            last_error = exc
            if attempt < MAX_RETRIES:
                delay = RETRY_BASE_DELAY ** (attempt - 1)
                log.warning(
                    "[%s] JSON parse error (attempt %d/%d), retrying in %.0fs: %s",
                    label,
                    attempt,
                    MAX_RETRIES,
                    delay,
                    exc,
                )
                await asyncio.sleep(delay)

        except Exception as exc:
            last_error = exc
            if attempt < MAX_RETRIES:
                delay = RETRY_BASE_DELAY ** (attempt - 1)
                log.warning(
                    "[%s] API error (attempt %d/%d), retrying in %.0fs: %s: %s",
                    label,
                    attempt,
                    MAX_RETRIES,
                    delay,
                    type(exc).__name__,
                    exc,
                )
                await asyncio.sleep(delay)
            else:
                raise

    raise ValueError(f"[{label}] Failed after {MAX_RETRIES} attempts. Last error: {last_error}")
