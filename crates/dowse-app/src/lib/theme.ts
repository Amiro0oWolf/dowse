export type Theme = 'light' | 'dark';

const THEME_KEY = 'dowse:theme';

export function loadTheme(): Theme {
	try {
		return localStorage.getItem(THEME_KEY) === 'dark' ? 'dark' : 'light';
	} catch {
		return 'light';
	}
}

export function applyTheme(theme: Theme): void {
	document.documentElement.dataset.theme = theme;
	try {
		localStorage.setItem(THEME_KEY, theme);
	} catch {
		// Keep the selected theme for this session if storage is unavailable.
	}
}
