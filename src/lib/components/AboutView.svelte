<!--
SPDX-License-Identifier: AGPL-3.0
Gosh Transfer - About View
-->
<script>
</script>

<div class="view-header">
  <h2 class="view-title">About</h2>
  <p class="view-subtitle">Information about Gosh Transfer</p>
</div>

<div class="card about-card">
  <div class="card-body">
    <div class="about-content">
      <div class="about-logo">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            d="M13 2L3 14h9l-1 8 10-12h-9l1-8z"
            stroke-linejoin="round"
            stroke-linecap="round"
          />
        </svg>
      </div>
      <h3 class="about-title">Gosh Transfer</h3>
      <p class="about-version">Version 2.0.1</p>
      <p class="about-description">
        Simple, explicit file transfers. No discovery magic, no cloud sync.
      </p>
      <p class="about-license">
        Licensed under AGPL-3.0
      </p>
    </div>
  </div>
</div>

<style>
  .about-card {
    max-width: 400px;
    margin: var(--space-6) auto;
    background-color: var(--bg-elevated);
  }

  .about-content {
    text-align: center;
    padding: var(--space-6);
  }

  .about-logo {
    display: flex;
    justify-content: center;
    margin-bottom: var(--space-4);
  }

  .about-logo svg {
    width: 64px;
    height: 64px;
    color: var(--primary);
  }

  .about-title {
    font-size: var(--font-size-xl);
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-1);
  }

  .about-version {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
    margin-bottom: var(--space-4);
  }

  .about-description {
    font-size: var(--font-size-base);
    color: var(--text-secondary);
    margin-bottom: var(--space-3);
  }

  .about-license {
    font-size: var(--font-size-sm);
    color: var(--text-muted);
  }
</style>
