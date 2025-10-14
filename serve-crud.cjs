#!/usr/bin/env node

const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

const PORT = 9000;
const DIST_DIR = path.join(__dirname, 'dist');

// MIME types
const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpg',
    '.gif': 'image/gif',
    '.ico': 'image/x-icon',
    '.svg': 'image/svg+xml',
    '.ttf': 'font/ttf',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2'
};

function getMimeType(filePath) {
    const ext = path.extname(filePath).toLowerCase();
    return mimeTypes[ext] || 'text/plain';
}

const server = http.createServer((req, res) => {
    console.log(`${new Date().toISOString()} - ${req.method} ${req.url}`);

    const parsedUrl = url.parse(req.url);
    let pathname = parsedUrl.pathname;

    // Security: prevent directory traversal
    if (pathname.includes('..')) {
        res.writeHead(403, { 'Content-Type': 'text/plain' });
        res.end('Forbidden');
        return;
    }

    // Default to index.html for root
    if (pathname === '/') {
        pathname = '/index.html';
    }

    // If no extension and not ending with /, try adding .html
    if (!path.extname(pathname) && !pathname.endsWith('/')) {
        pathname += '.html';
    }

    const filePath = path.join(DIST_DIR, pathname);

    fs.readFile(filePath, (err, data) => {
        if (err) {
            console.log(`File not found: ${filePath}`);
            res.writeHead(404, { 'Content-Type': 'text/html' });
            res.end(`
<!DOCTYPE html>
<html>
<head><title>404 - Not Found</title></head>
<body style="font-family: Arial, sans-serif; padding: 40px;">
    <h1>404 - File Not Found</h1>
    <p>The requested file <code>${pathname}</code> was not found.</p>
    <h2>Available CRUD Screens:</h2>
    <ul>
        <li><a href="/products-management.html">Products Management</a></li>
        <li><a href="/cbu-management.html">CBU Management</a></li>
        <li><a href="/resources-management.html">Resources Management</a></li>
        <li><a href="/schema.html">Schema Viewer</a></li>
    </ul>
    <p><a href="/">Back to Index</a></p>
</body>
</html>
            `);
            return;
        }

        const mimeType = getMimeType(filePath);
        res.writeHead(200, {
            'Content-Type': mimeType,
            'Cache-Control': 'no-cache'
        });
        res.end(data);
    });
});

server.listen(PORT, () => {
    console.log(`🚀 CRUD Screens Server running at http://localhost:${PORT}`);
    console.log(`📁 Serving files from: ${DIST_DIR}`);
    console.log(`\n📦 Available CRUD Screens:`);
    console.log(`   • Products Management: http://localhost:${PORT}/products-management.html`);
    console.log(`   • CBU Management: http://localhost:${PORT}/cbu-management.html`);
    console.log(`   • Resources Management: http://localhost:${PORT}/resources-management.html`);
    console.log(`   • Schema Viewer: http://localhost:${PORT}/schema.html`);
    console.log(`\n🛑 Press Ctrl+C to stop the server`);
});

process.on('SIGINT', () => {
    console.log('\n🛑 Server stopped');
    process.exit(0);
});