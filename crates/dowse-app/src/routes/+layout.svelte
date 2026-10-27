<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { getConfig } from '$lib/api';
	import { initializeLanguage } from '$lib/i18n';
	import { applyTheme, loadTheme } from '$lib/theme';

	let { children } = $props();
	let languageReady = $state(false);

	onMount(() => {
		applyTheme(loadTheme());
		getConfig()
			.then((config) => initializeLanguage(config.lang))
			.catch((error) => console.error('Could not load startup language', error))
			.finally(() => { languageReady = true; });
	});
</script>

{#if languageReady}
	{@render children()}
{/if}
