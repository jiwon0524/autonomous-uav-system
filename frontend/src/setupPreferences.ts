export type Theme = 'dark' | 'day';
export type GuideLanguage = 'ko' | 'en';
const completionKey = 'firescene.setup.complete.v1';
const themeKey = 'firescene.ui.theme';
const languageKey = 'firescene.setup.guideLanguage';
export function initialTheme(): Theme {
  try { return localStorage.getItem(themeKey) === 'day' ? 'day' : 'dark'; } catch { return 'dark'; }
}
export function saveTheme(theme: Theme) { try { localStorage.setItem(themeKey,theme); } catch { /* use current session */ } }
export function needsFirstRun(): boolean { try { return localStorage.getItem(completionKey) !== 'yes'; } catch { return true; } }
export function finishFirstRun(language: GuideLanguage, theme: Theme): boolean {
  try { localStorage.setItem(languageKey,language); localStorage.setItem(themeKey,theme); localStorage.setItem(completionKey,'yes'); return true; } catch { return false; }
}
