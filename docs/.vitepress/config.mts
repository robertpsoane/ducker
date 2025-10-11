import { defineConfig } from 'vitepress'

import { generateSidebar } from 'vitepress-sidebar';

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "🦆 Ducker",
  titleTemplate: "Robert Soane",
  description: "A slightly quackers Docker TUI based on k9s 🦆",
  lastUpdated: true,

  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config

    nav: [
      { text: 'Docs', link: '/docs/' },
      { text: 'About', link: 'https://soane.io/blog/2025/10/11/ducker/' },
    ],

    sidebar: generateSidebar({
      useTitleFromFileHeading: true,
      capitalizeFirst: true
    }),

    socialLinks: [
      { icon: 'github', link: 'https://github.com/robertpsoane/ducker' }
    ],
    footer: {
      message: "🦀 It's written in Rust so it must be good! 🦀",
    }
  }
})
