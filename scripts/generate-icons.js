import sharp from 'sharp';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import pngToIco from 'png-to-ico';
import png2icons from 'png2icons';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROJECT_ROOT = path.resolve(__dirname, '..');

const DEFAULT_CONFIG = {
  sourceImage: path.join(PROJECT_ROOT, 'opifex_icon.png'),
  iconsDir: path.join(PROJECT_ROOT, 'src-tauri', 'icons'),
  generateTauri: true,
  generateWeb: false
};

const TAURI_PNG_SIZES = [
  { size: 32, name: '32x32.png' },
  { size: 64, name: '64x64.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' },
  { size: 512, name: 'icon.png' },
];

const TAURI_SQUARE_SIZES = [
  { size: 30, name: 'Square30x30Logo.png' },
  { size: 44, name: 'Square44x44Logo.png' },
  { size: 71, name: 'Square71x71Logo.png' },
  { size: 89, name: 'Square89x89Logo.png' },
  { size: 107, name: 'Square107x107Logo.png' },
  { size: 142, name: 'Square142x142Logo.png' },
  { size: 150, name: 'Square150x150Logo.png' },
  { size: 284, name: 'Square284x284Logo.png' },
  { size: 310, name: 'Square310x310Logo.png' },
  { size: 50, name: 'StoreLogo.png' },
];

const TAURI_ANDROID_SIZES = [
  { size: 36, name: 'android/mipmap-mdpi/ic_launcher.png' },
  { size: 48, name: 'android/mipmap-hdpi/ic_launcher.png' },
  { size: 72, name: 'android/mipmap-xhdpi/ic_launcher.png' },
  { size: 96, name: 'android/mipmap-xxhdpi/ic_launcher.png' },
  { size: 144, name: 'android/mipmap-xxxhdpi/ic_launcher.png' },
];

const TAURI_IOS_SIZES = [
  { size: 40, name: 'ios/40.png' },
  { size: 60, name: 'ios/60.png' },
  { size: 58, name: 'ios/58.png' },
  { size: 87, name: 'ios/87.png' },
  { size: 80, name: 'ios/80.png' },
  { size: 120, name: 'ios/120.png' },
  { size: 180, name: 'ios/180.png' },
];

const TAURI_ICO_SIZES = [16, 32, 48, 64, 128, 256];

async function generateIcon(sourceImage, size, outputPath) {
  const dir = path.dirname(outputPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  await sharp(sourceImage)
    .resize(size, size, {
      fit: 'cover',
      position: 'center',
    })
    .png()
    .toFile(outputPath);

  console.log(`Generated: ${outputPath}`);
}

async function generateICO(sourceImage, sizes, outputPath) {
  try {
    const images = await Promise.all(
      sizes.map(async (size) => {
        return await sharp(sourceImage)
          .resize(size, size, {
            fit: 'cover',
            position: 'center',
          })
          .png()
          .toBuffer();
      })
    );

    const icoBuffer = await pngToIco(images);
    fs.writeFileSync(outputPath, icoBuffer);
    console.log(`Generated: ${outputPath}`);
  } catch (error) {
    console.error(`Error generating ICO: ${error.message}`);
    await sharp(sourceImage)
      .resize(64, 64, {
        fit: 'cover',
        position: 'center',
      })
      .toFile(outputPath.replace('.ico', '.png'));
    console.log(`Generated PNG fallback: ${outputPath.replace('.ico', '.png')}`);
  }
}

async function generateICNS(sourceImage, outputPath) {
  try {
    const tempPng = path.join(path.dirname(outputPath), 'temp_icns_source.png');
    await sharp(sourceImage)
      .resize(1024, 1024, {
        fit: 'cover',
        position: 'center',
      })
      .png()
      .toFile(tempPng);

    const pngBuffer = fs.readFileSync(tempPng);
    const icnsBuffer = png2icons.createICNS(pngBuffer);
    fs.writeFileSync(outputPath, icnsBuffer);
    fs.unlinkSync(tempPng);

    console.log(`Generated: ${outputPath}`);
  } catch (error) {
    console.error(`Error generating ICNS: ${error.message}`);
    console.log('ICNS generation failed - may require macOS or specialized tools');
  }
}

async function generateTauriIcons(sourceImage, iconsDir) {
  console.log('Generating Tauri application icons...');

  console.log('\nGenerating PNG icons...');
  for (const { size, name } of TAURI_PNG_SIZES) {
    await generateIcon(sourceImage, size, path.join(iconsDir, name));
  }

  console.log('\nGenerating square logos...');
  for (const { size, name } of TAURI_SQUARE_SIZES) {
    await generateIcon(sourceImage, size, path.join(iconsDir, name));
  }

  console.log('\nGenerating Android icons...');
  for (const { size, name } of TAURI_ANDROID_SIZES) {
    await generateIcon(sourceImage, size, path.join(iconsDir, name));
  }

  console.log('\nGenerating iOS icons...');
  for (const { size, name } of TAURI_IOS_SIZES) {
    await generateIcon(sourceImage, size, path.join(iconsDir, name));
  }

  console.log('\nGenerating ICO files...');
  await generateICO(sourceImage, TAURI_ICO_SIZES, path.join(iconsDir, 'icon.ico'));
  await generateICO(sourceImage, TAURI_ICO_SIZES, path.join(iconsDir, 'app.ico'));

  console.log('\nGenerating ICNS file...');
  await generateICNS(sourceImage, path.join(iconsDir, 'icon.icns'));

  console.log('\nTauri icon generation complete!');
}

async function main() {
  const args = process.argv.slice(2);
  let config = DEFAULT_CONFIG;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--source' && args[i + 1]) {
      config.sourceImage = path.resolve(args[i + 1]);
      i++;
    } else if (args[i] === '--output' && args[i + 1]) {
      config.iconsDir = path.resolve(args[i + 1]);
      i++;
    } else if (args[i] === '--tauri') {
      config = { ...config, generateTauri: true, generateWeb: false };
    } else if (args[i] === '--web') {
      config = { ...config, generateTauri: false, generateWeb: true };
    }
  }

  console.log('Starting icon generation...');
  console.log(`Source image: ${config.sourceImage}`);
  console.log(`Output directory: ${config.iconsDir}`);

  if (!fs.existsSync(config.sourceImage)) {
    console.error(`Source image not found: ${config.sourceImage}`);
    process.exit(1);
  }

  if (!fs.existsSync(config.iconsDir)) {
    fs.mkdirSync(config.iconsDir, { recursive: true });
  }

  if (config.generateTauri) {
    await generateTauriIcons(config.sourceImage, config.iconsDir);
  }

  console.log('\nIcon generation complete!');
}

main().catch(console.error);
