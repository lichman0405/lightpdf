import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'LightPDF',
  description: 'A fast, lightweight PDF toolkit — desktop app + MCP server',
  base: '/lightpdf/',

  head: [
    ['link', { rel: 'icon', href: '/lightpdf/favicon.ico' }],
  ],

  themeConfig: {
    logo: '/logo.svg',
    siteTitle: 'LightPDF',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Architecture', link: '/architecture/overview' },
      {
        text: 'GitHub',
        link: 'https://github.com/lichman0405/lightpdf',
      },
    ],

    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'Home', link: '/' },
        ],
      },
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Desktop App', link: '/guide/gui' },
          { text: 'MCP Server', link: '/guide/mcp-server' },
        ],
      },
      {
        text: 'Architecture',
        items: [
          { text: 'Overview', link: '/architecture/overview' },
          { text: 'Core Library', link: '/architecture/core' },
          { text: 'Annotation System', link: '/architecture/annotations' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/lichman0405/lightpdf' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2026 lichman0405',
    },

    search: {
      provider: 'local',
    },
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark: 'github-dark',
    },
    // Enable mermaid diagrams via html: true
    html: true,
  },
})
