require('../patches/fix-pglite-prisma-bytes.cjs');

if (process.env.SKIP_VIBE_APP_PATCHES === '1') {
  console.log('[postinstall] SKIP_VIBE_APP_PATCHES=1, skipping Vibe app patch bootstrap');
  process.exit(0);
}

console.log('[postinstall] Applied Vibe app bootstrap patches');
