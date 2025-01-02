import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import cloudflare from '@astrojs/cloudflare';
import starlightDocSearch from '@astrojs/starlight-docsearch';

// https://astro.build/config
export default defineConfig({
  output: 'server',
  site: 'https://igit.erguotou.me',
  vite: {
    ssr: {
      // noExternal: ['@astrojs/starlight'],
      external: ['node:path', 'node:url']
    }
  },
	integrations: [
		starlight({
			title: {
				en: 'iGit Docs',
				'zh-CN': 'iGit 文档'
			},
			defaultLocale: 'root',
			locales: {
				root: {
					label: 'English',
          lang: 'en',
				},
				'zh-cn': {
					label: '简体中文',
					lang: 'zh-CN',
				},
			},
			social: {
				github: 'https://github.com/doremijs/igit',
			},
			sidebar: [
				{
					label: 'Guides',
					translations: {
						'zh-CN': '指南',
					},
					items: [
						{
              label: 'Getting Started',
              slug: 'guides/getting-started',
              translations: {
                'zh-CN': '快速开始',
              }
            },
            {
              label: 'Hooks',
              slug: 'guides/hooks',
              translations: {
                'zh-CN': '钩子',
              }
            },
            {
              label: 'ai-commit',
              slug: 'guides/ai-commit',
              translations: {
                'zh-CN': 'AI 提交',
              }
            },
            {
              label: 'Configuration',
              slug: 'guides/configuration',
              translations: {
                'zh-CN': '配置',
              }
            }
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
					translations: {
						'zh-CN': '参考',
					},
				}
			],
      plugins: [
        starlightDocSearch({
          appId: 'H73GHR1OE5',
          apiKey: process.env.DOCSEARCH_API_KEY,
          indexName: 'igit-erguotou',
        }),
      ],
		}),
	],
	adapter: cloudflare({
		// imageService: 'cloudflare',
	}),
});
