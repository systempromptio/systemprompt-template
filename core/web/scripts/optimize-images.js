#!/usr/bin/env node

/**
 * Image Optimization Utility
 *
 * Converts JPEG and PNG images to WebP format for improved compression and performance.
 * Automatically processes all images in src/assets/images directory.
 *
 * WebP compression benefits:
 * - 25-35% smaller file sizes than JPEG
 * - 25-35% smaller file sizes than PNG (with lossy mode)
 * - Universal browser support (with fallbacks via <picture> tag)
 * - Maintains quality at high compression ratios
 *
 * Configuration:
 * - Quality: 80 (balanced between compression and visual quality)
 * - Alpha Quality: 100 (preserve transparency perfectly)
 * - Method: 6 (slower but better compression)
 *
 * @example
 * ```bash
 * npm run images:optimize
 * ```
 *
 * Output includes:
 * - File count and sizes
 * - Format distribution
 * - Best practice recommendations
 * - Exit code 0 on success, 1 on error
 */

import { fileURLToPath } from 'url'
import { dirname, join } from 'path'
import imagemin from 'imagemin'
import imageminWebp from 'imagemin-webp'
import fs from 'fs'

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)
const ROOT_DIR = join(__dirname, '..')
const IMAGES_DIR = join(ROOT_DIR, 'src', 'assets', 'images')

const DIVIDER = '='.repeat(50)

/**
 * Recursively collect image file extensions from directory
 *
 * Walks the entire directory tree and counts file extensions,
 * returning a map of extension -> count for summary reporting.
 *
 * @param {string} directoryPath - Path to directory to scan
 * @returns {Object.<string, number>} Map of file extensions to counts
 * @throws {Error} If directory cannot be read
 */
function countImagesByExtension(directoryPath) {
  const extensionCounts = {}

  const walkDirectory = (currentPath) => {
    try {
      const items = fs.readdirSync(currentPath)
      items.forEach((item) => {
        const itemPath = join(currentPath, item)
        const itemStats = fs.statSync(itemPath)

        if (itemStats.isDirectory()) {
          walkDirectory(itemPath)
        } else {
          const extension = item.split('.').pop()?.toLowerCase() || 'unknown'
          extensionCounts[extension] = (extensionCounts[extension] || 0) + 1
        }
      })
    } catch (error) {
      throw new Error(`Failed to read directory ${currentPath}: ${error.message}`)
    }
  }

  walkDirectory(directoryPath)
  return extensionCounts
}

/**
 * Convert bytes to human-readable format
 *
 * Automatically selects appropriate unit (Bytes, KB, MB, GB).
 * Rounds to 2 decimal places for readability.
 *
 * @param {number} bytes - Size in bytes
 * @returns {string} Formatted size string (e.g., "1.5 MB")
 *
 * @example
 * ```js
 * formatBytes(1536) // Returns "1.5 KB"
 * formatBytes(1048576) // Returns "1 MB"
 * ```
 */
function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes'

  const unitSize = 1024
  const units = ['Bytes', 'KB', 'MB', 'GB']
  const exponent = Math.floor(Math.log(bytes) / Math.log(unitSize))
  const value = bytes / Math.pow(unitSize, exponent)

  return `${Math.round(value * 100) / 100} ${units[exponent]}`
}

/**
 * Display optimization summary with recommendations
 *
 * Prints formatted summary including:
 * - File format distribution
 * - Optimization statistics
 * - Best practice recommendations for production
 *
 * @returns {void}
 */
function displayOptimizationSummary() {
  console.log('\n📊 Image Optimization Summary:')
  console.log(DIVIDER)

  const extensionCounts = countImagesByExtension(IMAGES_DIR)

  console.log('File format distribution:')
  Object.entries(extensionCounts)
    .sort((a, b) => b[1] - a[1])
    .forEach(([extension, count]) => {
      console.log(`  ${extension.toUpperCase()}: ${count} file(s)`)
    })

  console.log('\n✅ Next steps for production:')
  console.log('  1. Test WebP images in all target browsers')
  console.log('  2. Implement <picture> tag for progressive image loading')
  console.log('  3. Add responsive sizes with srcset attribute')
  console.log('  4. Enable lazy loading with loading="lazy"')
  console.log('  5. Monitor Core Web Vitals in production')
  console.log(DIVIDER)
}

/**
 * Convert images to WebP format for improved performance
 *
 * Processes all JPEG and PNG images in IMAGES_DIR, converting
 * them to WebP format alongside original files. Original files
 * are preserved for fallback support.
 *
 * Process:
 * 1. Validates images directory exists
 * 2. Converts supported formats to WebP
 * 3. Reports results and recommendations
 * 4. Exits with appropriate status code
 *
 * @async
 * @returns {Promise<void>}
 * @throws {Error} If optimization fails unexpectedly
 */
async function optimizeImages() {
  console.log('🖼️  Starting image optimization...')

  if (!fs.existsSync(IMAGES_DIR)) {
    console.error(`❌ Images directory not found: ${IMAGES_DIR}`)
    process.exit(1)
  }

  try {
    console.log('📦 Converting images to WebP format...')

    const optimizedFiles = await imagemin(
      [`${IMAGES_DIR}/**/*.{jpg,jpeg,png}`],
      {
        destination: IMAGES_DIR,
        plugins: [
          imageminWebp({
            quality: 80,
            alphaQuality: 100,
            method: 6,
          }),
        ],
        glob: {
          ignore: ['**/*.webp'],
        },
      }
    )

    if (optimizedFiles.length > 0) {
      console.log(`✅ Successfully optimized ${optimizedFiles.length} images`)
      optimizedFiles.forEach((file) => {
        try {
          const size = fs.statSync(file).size
          console.log(`   • ${file} (${formatBytes(size)})`)
        } catch (error) {
          console.warn(`   • ${file} (size unavailable)`)
        }
      })
    } else {
      console.log('ℹ️  No new images to optimize')
    }

    displayOptimizationSummary()
    process.exit(0)
  } catch (error) {
    console.error('❌ Optimization failed')
    console.error(
      `   Error: ${error instanceof Error ? error.message : String(error)}`
    )
    process.exit(1)
  }
}

optimizeImages()
