module.exports = {
  mode: "jit",
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  darkMode: "class", // or 'media' or 'class'
  theme: {
    extend: {
      scale: {
        101: "1.01",
      },
    },
  },
  variants: {
    extend: { ringWidth: ["hover", "active"] },
  },
  plugins: [
    require("@tailwindcss/line-clamp"),
    require("@kamona/tailwindcss-perspective"),
  ],
};
