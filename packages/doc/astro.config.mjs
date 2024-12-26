// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import cloudflare from '@astrojs/cloudflare';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'iGit Docs',
			social: {
				github: 'https://github.com/doremijs/igit',
			},
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Guide', slug: 'guides/get-started' },
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
  adapter: cloudflare({
    imageService: 'cloudflare'
  })
});
