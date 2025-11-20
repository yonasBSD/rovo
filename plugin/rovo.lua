-- Rovo plugin initialization
-- Ensures syntax highlighting is loaded for Rust files

-- Create an autocommand group for Rovo
local group = vim.api.nvim_create_augroup('Rovo', { clear = true })

-- Setup syntax highlighting when Rust files are opened
vim.api.nvim_create_autocmd('FileType', {
  group = group,
  pattern = 'rust',
  callback = function(args)
    -- Only setup if not already done
    if vim.b[args.buf].rovo_syntax_loaded then
      return
    end
    vim.b[args.buf].rovo_syntax_loaded = true

    -- Define highlight groups with fallbacks
    local highlights = {
      RovoAnnotation = { fg = '#C678DD', bold = true },
      RovoStatusCode = { fg = '#61AFEF' },
      RovoResponseType = { fg = '#56B6C2' },
      RovoSecurityScheme = { fg = '#E5C07B' },
    }

    for name, hl in pairs(highlights) do
      vim.api.nvim_set_hl(0, name, vim.tbl_extend('force', hl, { default = true }))
    end
  end,
})
