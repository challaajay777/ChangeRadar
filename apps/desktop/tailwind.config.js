/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        cr: {
          bg: '#101114',
          surface: '#181A20',
          raised: '#20232B',
          border: '#2C303A',
          hover: '#252933',
          text: '#F2F3F5',
          muted: '#A1A6B2',
          accent: '#6C8CFF',
          'accent-hover': '#5876E8',
          success: '#42C98A',
          warning: '#E9B65B',
          danger: '#EA6868',
        }
      }
    },
  },
  plugins: [],
}
