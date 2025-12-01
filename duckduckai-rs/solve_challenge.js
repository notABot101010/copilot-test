const crypto = require('crypto');

// Create a proper HTMLCollection-like object
function createNodeList(length = 0) {
    const list = [];
    for (let i = 0; i < length; i++) {
        list.push({ tagName: 'DIV' });
    }
    list.item = (i) => list[i];
    return list;
}

// Mock element class
function createElement(tag) {
    const elem = {
        tagName: tag.toUpperCase(),
        innerHTML: '',
        innerText: '',
        textContent: '',
        srcdoc: '',
        sandbox: '',
        children: createNodeList(0),
        getAttribute: (name) => null,
        setAttribute: (name, value) => {},
        querySelector: (sel) => null,
        querySelectorAll: (sel) => {
            // Return a mock NodeList with 0 elements
            return createNodeList(0);
        },
        appendChild: (child) => child,
        removeChild: (child) => child,
        contentWindow: {
            self: {
                get: undefined  // This is what the bot check looks for
            },
            document: {}
        },
        contentDocument: {}
    };
    
    // For div elements that get innerHTML set, we need to track it
    Object.defineProperty(elem, 'innerHTML', {
        get: function() { return this._innerHTML || ''; },
        set: function(val) { 
            this._innerHTML = val;
            // Parse the innerHTML and update children count
            const tagCount = (val.match(/<[a-z]+/gi) || []).length;
            this.children = createNodeList(tagCount);
        }
    });
    
    return elem;
}

// Simulated browser environment
global.navigator = {
    userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36',
    language: 'en-US',
    languages: ['en-US', 'en'],
    platform: 'MacIntel',
    webdriver: false,
    cookieEnabled: true,
    onLine: true
};

global.document = {
    body: {
        appendChild: (child) => child,
        removeChild: (child) => child,
        onerror: null
    },
    head: {
        appendChild: (child) => child
    },
    createElement: createElement,
    documentElement: {
        querySelector: (sel) => null
    },
    domain: 'duckduckgo.com',
    referrer: ''
};

global.window = {
    top: null,
    self: null,
    parent: null,
    location: { 
        origin: 'https://duckduckgo.com',
        href: 'https://duckduckgo.com/aichat',
        protocol: 'https:',
        host: 'duckduckgo.com',
        hostname: 'duckduckgo.com'
    },
    navigator: global.navigator,
    document: global.document,
    innerWidth: 1920,
    innerHeight: 1080,
    screen: { width: 1920, height: 1080, colorDepth: 24 },
    JSON: JSON,
    Object: Object,
    Array: Array,
    Promise: Promise,
    String: String,
    Number: Number,
    Symbol: Symbol,
    Proxy: Proxy,
    hasOwnProperty: (prop) => false
};
global.window.top = global.window;
global.window.self = global.window;
global.window.parent = global.window;

global.self = global.window;
global.top = global.window;
global.parent = global.window;
global.location = global.window.location;

// Get challenge from stdin
let inputData = '';
process.stdin.on('data', (chunk) => {
    inputData += chunk;
});

process.stdin.on('end', async () => {
    try {
        const challengeB64 = inputData.trim();
        const challengeCode = Buffer.from(challengeB64, 'base64').toString('utf8');
        
        // Execute the challenge
        const result = await eval(challengeCode);
        
        // Hash the client_hashes with SHA-256 and encode as base64
        const hashedClientHashes = result.client_hashes.map((hash) => {
            const hashBuffer = crypto.createHash('sha256').update(hash).digest();
            return hashBuffer.toString('base64');
        });
        
        // Build the final result
        const finalResult = {
            server_hashes: result.server_hashes,
            client_hashes: hashedClientHashes,
            signals: result.signals || {},
            meta: {
                ...result.meta,
                origin: 'https://duckduckgo.com',
                stack: 'Error\nat l (https://duckduckgo.com/dist/wpm.main.758b58e5295173a9d89c.js:1:424103)',
                duration: String(Math.floor(Math.random() * 20) + 5)
            }
        };
        
        // Output as base64 encoded JSON
        console.log(Buffer.from(JSON.stringify(finalResult)).toString('base64'));
    } catch (err) {
        console.error('Error:', err.message);
        console.error(err.stack);
        process.exit(1);
    }
});
