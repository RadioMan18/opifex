module.exports = {
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        aether: {
          bg: 'var(--aether-bg)',
          surface: 'var(--aether-surface)',
          'surface-border': 'var(--aether-border)',
          text: 'var(--aether-text)',
          'text-muted': 'var(--aether-text-muted)',
          
          // Action (Blue)
          primary: 'var(--aether-primary)',
          'primary-hover': 'var(--aether-primary-hover)',
          'primary-fg': 'var(--aether-primary-fg)',

          // Selectable & Retained State (Green)
          selected: 'var(--aether-selected)',
          'selected-border': 'var(--aether-selected-border)',
          'selected-fg': 'var(--aether-selected-fg)',

          // Semantics
          success: 'var(--aether-success)',
          warning: 'var(--aether-warning)',
          error: 'var(--aether-error)',
          info: 'var(--aether-info)',
        },
      },
    },
  },
};