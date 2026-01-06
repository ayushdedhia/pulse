import pngToIco from 'png-to-ico';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, '../src-tauri/icons');

async function generateIco() {
  const pngPath = path.join(iconsDir, 'icon.png');
  const icoPath = path.join(iconsDir, 'icon.ico');

  const ico = await pngToIco(pngPath);
  fs.writeFileSync(icoPath, ico);
  console.log('Generated proper icon.ico');
}

generateIco().catch(console.error);
