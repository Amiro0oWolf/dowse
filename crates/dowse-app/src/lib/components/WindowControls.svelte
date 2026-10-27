<script lang="ts">
	import * as api from '$lib/api';
	import { t } from '$lib/i18n';

	let maximized = $state(false);

	function minimize() {
		api.minimizeWindow().catch((err) => console.error('minimizeWindow failed', err));
	}

	async function toggleMaximize() {
		try {
			maximized = await api.toggleMaximize();
		} catch (err) {
			console.error('toggleMaximize failed', err);
		}
	}

	function exit() {
		api.exitApp().catch((err) => console.error('exitApp failed', err));
	}
</script>

<div class="window-controls" data-no-window-drag>
	<button type="button" class="window-control" title={t.windowMinimize} aria-label={t.windowMinimize} onclick={minimize}>
		<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10" /></svg>
	</button>
	<button
		type="button"
		class="window-control"
		title={maximized ? t.windowRestore : t.windowMaximize}
		aria-label={maximized ? t.windowRestore : t.windowMaximize}
		onclick={toggleMaximize}
	>
		{#if maximized}
			<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M5 3.5h7.5V11M3.5 5H11v7.5H3.5z" /></svg>
		{:else}
			<svg viewBox="0 0 16 16" aria-hidden="true"><rect x="3.5" y="3.5" width="9" height="9" /></svg>
		{/if}
	</button>
	<button type="button" class="window-control close" title={t.windowExit} aria-label={t.windowExit} onclick={exit}>
		<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m4 4 8 8m0-8-8 8" /></svg>
	</button>
</div>

<style>
	.window-controls {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-left: 4px;
		padding-left: 8px;
		border-left: 1px solid var(--divider);
		flex-shrink: 0;
	}

	.window-control {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 26px;
		padding: 0;
		border: 0;
		border-radius: var(--radius-chip);
		background: transparent;
		color: var(--fg-secondary);
		cursor: default;
	}

	.window-control:hover {
		background: var(--row-hover);
	}

	.window-control.close:hover {
		background: #c42b1c;
		color: white;
	}

	svg {
		width: 14px;
		height: 14px;
		fill: none;
		stroke: currentColor;
		stroke-width: 1.2;
		stroke-linecap: round;
		stroke-linejoin: round;
	}
</style>
