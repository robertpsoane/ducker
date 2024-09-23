import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Ducker",
  description: "🦆 A terminal app for managing docker containers, inpsired by K9s.",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Examples', link: '/docs/markdown-examples' }
    ],

    // sidebar: [
    //   {
    //     text: 'Examples',
    //     items: [
    //       { text: 'Markdown Examples', link: '/markdown-examples' },
    //       { text: 'Runtime API Examples', link: '/api-examples' }
    //     ]
    //   }
    // ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/robertpsoane/ducker' }
    ],

    footer: {
      message: "🦀 It's written in rust so it must be good! 🦀",
      copyright: "Copyright © 2024-present Robert Soane.  Released under MIT License."
    }
  }
})
