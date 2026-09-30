import { noRawColors } from './no-raw-colors.js';
import { typescriptRuleTester } from './rule-tester.js';

const filename = 'packages/ui/src/components/button.tsx';
const rawColor = [{ messageId: 'rawColor' }];

function valid(code, options = {}) {
  return { filename, ...options, code };
}

function invalid(code, count = 1) {
  return { code, filename, errors: Array.from({ length: count }, () => rawColor[0]) };
}

typescriptRuleTester().run('no-raw-colors', noRawColors, {
  valid: [
    valid('const color = "var(--ramal-accent)";'),
    valid('const title = "Fix #123";'),
    valid('const title = `Merge pull request #4567 from org/rama`;'),
    valid('const url = "https://github.com/org/repo#readme";'),
    valid('const anchor = "docs#abc-def";'),
    valid('const entity = "&#123;";'),
    valid('const version = "v1.2.3";'),
    valid('const label = "GitLab(beta)";'),
    valid('export const accent = "#7aa2f7";', { filename: 'packages/ui/src/themes/dark.ts' }),
    valid('export const accent = "oklch(0.7 0.1 250)";', {
      filename: '/repo/packages/ui/src/themes/light.ts',
    }),
    valid('export const accent = "#fff";', {
      filename: 'apps/web/src/palette/colors.ts',
      options: [{ allow: ['apps/web/src/palette/'] }],
    }),
  ],
  invalid: [
    invalid('const color = "#fff";'),
    invalid('const color = "#1e1e2e";'),
    invalid('const color = "#1e1e2eff";'),
    invalid('const color = "#333";'),
    invalid('const border = "1px solid #333";'),
    invalid('const color = "oklch(0.7 0.1 250)";'),
    invalid('const color = "rgb(0 0 0 / 50%)";'),
    invalid('const color = "rgba(0, 0, 0, 0.5)";'),
    invalid('const color = "hsl(200 50% 50%)";'),
    invalid('const style = { color: "#fafafa", background: "#000000" };', 2),
    invalid('const a = <rect fill="#abc" />;'),
    invalid('const css = `color: ${fg}; border-color: #abcdef;`;'),
    invalid('const shadow = `0 0 4px rgb(0 0 0)`;'),
  ],
});
