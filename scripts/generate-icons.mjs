import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, '../src-tauri/icons');

// Ensure icons directory exists
if (!fs.existsSync(iconsDir)) {
  fs.mkdirSync(iconsDir, { recursive: true });
}

// SVG source
const svgContent = `<svg width="512" height="512" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect width="512" height="512" rx="100" fill="#00A884"/>
  <path d="M256 88C162.4 88 88 162.4 88 256c0 37.2 12 71.6 32.4 99.6L96 424l71.6-23.6C193.6 418 223.6 428 256 428c93.6 0 168-74.4 168-168S349.6 88 256 88z" fill="white"/>
  <path d="M196 186c-5.6 0-11.6 2-16 6.4-5.2 5.2-8 12-8 19.2 0 7.6 2.4 14.8 7.2 22.4 9.6 15.2 25.2 33.6 44.8 52 19.6 18.4 41.6 34 59.2 42.4 8.8 4.2 16 6.4 22.8 6.4 6.8 0 13.2-2.4 18-7.2 4.8-4.8 7.2-11.2 7.2-18 0-2.8-.8-5.6-2-8l-24-40c-2.4-4-6.4-6.4-10.8-6.4-2.8 0-5.6.8-8 2.4l-12 8c-1.2.8-2.4 1.2-3.6 1.2-1.6 0-3.2-.8-4.4-2-8-7.2-15.2-15.2-21.6-24-1.2-1.6-2-3.2-2-4.8 0-1.2.4-2.4 1.2-3.6l8-12c1.6-2.4 2.4-5.2 2.4-8 0-4.4-2.4-8.4-6.4-10.8l-40-24c-2.4-1.2-5.2-2-8-2z" fill="#00A884"/>
</svg>`;

async function generateIcons() {
  const svgBuffer = Buffer.from(svgContent);

  // Generate PNG icons
  const sizes = [32, 128, 256];

  for (const size of sizes) {
    const filename = size === 256 ? '128x128@2x.png' : `${size}x${size}.png`;
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(path.join(iconsDir, filename));
    console.log(`Generated ${filename}`);
  }

  // Generate ICO (Windows) - use 256x256 PNG as base
  const png256 = await sharp(svgBuffer)
    .resize(256, 256)
    .png()
    .toBuffer();

  // For ICO, we need to create a multi-resolution icon
  // Simple approach: use png-to-ico or just use a single 256x256
  // Tauri actually accepts PNG files renamed to .ico on modern Windows
  await sharp(svgBuffer)
    .resize(256, 256)
    .png()
    .toFile(path.join(iconsDir, 'icon.png'));

  // Copy as ico (Windows 10+ accepts PNG in ICO)
  fs.copyFileSync(
    path.join(iconsDir, 'icon.png'),
    path.join(iconsDir, 'icon.ico')
  );
  console.log('Generated icon.ico');

  console.log('All icons generated successfully!');
}

generateIcons().catch(console.error);
