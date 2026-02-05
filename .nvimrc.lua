return {
	lsp_servers = { "markdown-oxide" },
	lsp_configs = {
		markdown_oxide = {
			filetypes = { "markdown", "markdown.mdx" },
		},
	},
	filetype_configs = {
		markdown = {
			opts = {
				textwidth = 80,
				wrap = true,
				linebreak = true,
				breakindent = true,
				showbreak = "↪ ",
				conceallevel = 2,
				spell = true,
				spelllang = "en_us",
			},
			commands = {
				"setlocal colorcolumn=",
			},
		},
	},
}
