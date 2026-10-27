<script lang="ts">
	import { t } from '$lib/i18n';
	import type { Theme } from '$lib/theme';

	let { theme, onclick }: { theme: Theme; onclick: () => void } = $props();
</script>

<button
	type="button"
	class="theme-button"
	title={theme === 'dark' ? t.themeLight : t.themeDark}
	aria-label={theme === 'dark' ? t.themeLight : t.themeDark}
	{onclick}
>
	{#if theme === 'dark'}
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<circle cx="12" cy="12" r="4" />
			<path d="M12 2v2m0 16v2M4.93 4.93l1.42 1.42m11.3 11.3 1.42 1.42M2 12h2m16 0h2M4.93 19.07l1.42-1.42m11.3-11.3 1.42-1.42" />
		</svg>
	{:else}
		<svg viewBox="0 0 24 24" aria-hidden="true">
			<path d="M20.5 15.3A8.5 8.5 0 0 1 8.7 3.5 8.5 8.5 0 1 0 20.5 15.3Z" />
		</svg>
	{/if}
</button>

<style>
	.theme-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		padding: 0;
		border: 0;
		border-radius: var(--radius-chip);
		background: transparent;
		color: var(--fg-tertiary);
		cursor: default;
	}

	.theme-button:hover {
		color: var(--fg-secondary);
		background: var(--row-hover);
	}

	svg {
		width: 15px;
		height: 15px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.6;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
