/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{astro,html,js,jsx,ts,tsx}'],
  theme: {
    extend: {
      fontFamily: {
        sans:     ['Plus Jakarta Sans', 'system-ui', 'sans-serif'],
        title:    ['Barlow Condensed', 'system-ui', 'sans-serif'],
        'mono-dm': ['DM Mono', 'monospace'],
      },
    },
  },
  plugins: [],
};
