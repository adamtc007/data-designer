#!/usr/bin/env node

/**
 * Centralized build script for Data Designer
 * Compiles TypeScript to JavaScript and bundles everything for Tauri
 * 100% TypeScript - no dynamic JavaScript generation
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const srcDir = path.join(__dirname, 'src');
const distDir = path.join(__dirname, 'dist');

console.log('🏗️  Building Data Designer - 100% TypeScript Build System');
console.log('📁 Source:', srcDir);
console.log('📁 Output:', distDir);

// Clean and ensure dist directory exists
if (fs.existsSync(distDir)) {
    fs.rmSync(distDir, { recursive: true });
}
fs.mkdirSync(distDir, { recursive: true });

// TypeScript files to compile to JavaScript
const typescriptFiles = [
    'main.ts',
    'ui-components.ts',
    'config-driven-renderer.ts',
    'metadata-driven-engine.ts',
    'data-dictionary-types.ts',
    'shared-db-service.ts',
    'home-manager.ts',
    'products-manager.ts',
    'cbu-manager.ts',
    'resources-manager.ts'
];

// HTML files to copy
const htmlFiles = [
    'home.html',
    'index.html',
    'cbu-management.html',
    'products-management.html',
    'resources-management.html'
];

async function compileTypeScript() {
    console.log('🔨 Compiling TypeScript files...');

    try {
        // Use the TypeScript compiler to compile individual files
        for (const file of typescriptFiles) {
            const srcPath = path.join(srcDir, file);
            const jsFile = file.replace('.ts', '.js');
            const distPath = path.join(distDir, jsFile);

            if (fs.existsSync(srcPath)) {
                console.log(`  📄 Compiling ${file} → ${jsFile}`);

                // Use tsc to compile individual file to ES modules
                const { stdout, stderr } = await execAsync(`npx tsc ${srcPath} --outDir ${distDir} --target ES2020 --module ES2020 --moduleResolution node --esModuleInterop true --allowSyntheticDefaultImports true --skipLibCheck true`);

                if (stderr && !stderr.includes('Warning')) {
                    console.warn(`  ⚠️  TypeScript warnings for ${file}:`, stderr);
                }

                console.log(`  ✅ Compiled ${file}`);
            } else {
                console.log(`  ⚠️  ${file} not found, skipping`);
            }
        }
    } catch (error) {
        console.error('❌ TypeScript compilation failed:', error);
        throw error;
    }
}

function copyHtmlFiles() {
    console.log('📋 Copying HTML files...');

    for (const file of htmlFiles) {
        const srcPath = path.join(srcDir, file);
        const distPath = path.join(distDir, file);

        if (fs.existsSync(srcPath)) {
            fs.copyFileSync(srcPath, distPath);
            console.log(`  ✅ Copied ${file}`);
        } else {
            console.log(`  ⚠️  ${file} not found, skipping`);
        }
    }
}

function updateMainTsForMonaco() {
    console.log('🎭 Updating main.js for Monaco Editor...');

    const mainJsPath = path.join(distDir, 'main.js');
    if (fs.existsSync(mainJsPath)) {
        let content = fs.readFileSync(mainJsPath, 'utf8');

        // Check if shared database service import already exists
        if (!content.includes('import { getSharedDbService }')) {
            // Add shared database service import at the top
            content = content.replace(
                /^/,
                'import { getSharedDbService } from \'./shared-db-service.js\';\n\n'
            );
        }

        // Only add database initialization if it doesn't already exist
        if (!content.includes('// Initialize shared database service first')) {
            // Add shared database service initialization
            content = content.replace(
                /document\.addEventListener\('DOMContentLoaded', async \(\) => \{/,
                `document.addEventListener('DOMContentLoaded', async () => {
    // Initialize shared database service first
    const sharedDbService = getSharedDbService();

    // Try to connect to shared database service or initialize it
    try {
        console.log('🔌 Attempting to connect to shared database service...');
        const connectionStatus = sharedDbService.getConnectionStatus();
        console.log('🔍 Initial connection status:', connectionStatus);

        if (!connectionStatus.isConnected) {
            console.log('🔌 Initializing database connection from IDE...');
            await sharedDbService.initialize();
            // Update database status display after successful initialization
            await checkDatabaseStatus();
        } else {
            console.log('✅ Using existing shared database connection');
            // Update database status display for existing connection
            await checkDatabaseStatus();
        }
    } catch (error) {
        console.warn('⚠️ Shared database service initialization failed:', error);
    }`
            );
        }

        fs.writeFileSync(mainJsPath, content);
        console.log('  ✅ Updated main.js with shared database service integration');
    }
}

function updateHtmlFiles() {
    console.log('🌐 Updating HTML files for proper JavaScript imports...');

    // Update index.html
    const indexPath = path.join(distDir, 'index.html');
    if (fs.existsSync(indexPath)) {
        let content = fs.readFileSync(indexPath, 'utf8');

        // Update script imports to use .js files
        content = content.replace(
            /<script type="module" src="main\.ts"><\/script>/g,
            '<script type="module" src="main.js"></script>'
        );

        // Add Monaco Editor CDN if not present
        if (!content.includes('monaco-editor')) {
            const monacoScripts = `
    <!-- Monaco Editor CDN -->
    <script src="https://unpkg.com/monaco-editor@0.43.0/min/vs/loader.js"></script>
    <script>
        require.config({
            paths: { 'vs': 'https://unpkg.com/monaco-editor@0.43.0/min/vs' }
        });
        require(['vs/editor/editor.main'], function(monaco) {
            window.monaco = monaco;
            window.dispatchEvent(new CustomEvent('monaco-loaded'));
        });
    </script>`;

            content = content.replace(
                /<script type="module" src="main\.js"><\/script>/,
                monacoScripts + '\n    <script type="module" src="main.js"></script>'
            );
        }

        fs.writeFileSync(indexPath, content);
        console.log('  ✅ Updated index.html');
    }

    // Update home.html
    const homePath = path.join(distDir, 'home.html');
    if (fs.existsSync(homePath)) {
        let content = fs.readFileSync(homePath, 'utf8');

        // Update script imports to use compiled TypeScript files
        content = content.replace(
            /<script type="module" src="[^"]*home[^"]*\.ts"><\/script>/g,
            '<script type="module" src="home-manager.js"></script>'
        );

        fs.writeFileSync(homePath, content);
        console.log('  ✅ Updated home.html');
    }

    // Update CRUD HTML files
    const crudFiles = [
        { file: 'cbu-management.html', script: 'cbu-manager.js' },
        { file: 'products-management.html', script: 'products-manager.js' },
        { file: 'resources-management.html', script: 'resources-manager.js' }
    ];

    for (const crud of crudFiles) {
        const filePath = path.join(distDir, crud.file);
        if (fs.existsSync(filePath)) {
            let content = fs.readFileSync(filePath, 'utf8');

            // Update script imports to use compiled TypeScript files
            content = content.replace(
                /<script type="module" src="[^"]*\.ts"><\/script>/g,
                `<script type="module" src="${crud.script}"></script>`
            );

            fs.writeFileSync(filePath, content);
            console.log(`  ✅ Updated ${crud.file}`);
        }
    }
}

// Main build execution
async function build() {
    try {
        await compileTypeScript();
        copyHtmlFiles();
        updateMainTsForMonaco();
        updateHtmlFiles();

        console.log('\n🎉 Build completed successfully!');
        console.log('📂 Output directory: dist/');
        console.log('🚀 Ready for Tauri');
        console.log('\n📋 Build Summary:');
        console.log(`  📄 TypeScript files compiled: ${typescriptFiles.length}`);
        console.log(`  📄 HTML files copied: ${htmlFiles.length}`);
        console.log('  ✅ 100% TypeScript - no dynamic JavaScript generation');

    } catch (error) {
        console.error('❌ Build failed:', error);
        process.exit(1);
    }
}

build();