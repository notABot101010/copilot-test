import { h, render, Fragment } from 'https://esm.sh/preact@10.19.3';
import { useState, useEffect, useMemo, useCallback } from 'https://esm.sh/preact@10.19.3/hooks';
import { signal, effect, computed } from 'https://esm.sh/@preact/signals@1.2.2';
import htm from 'https://esm.sh/htm@3.1.1';

// Initialize htm with Preact
const html = htm.bind(h);

// Simple signal-based router
const currentPath = signal(window.location.pathname);
const currentQuery = signal(new URLSearchParams(window.location.search));

// Listen for popstate events
window.addEventListener('popstate', () => {
    currentPath.value = window.location.pathname;
    currentQuery.value = new URLSearchParams(window.location.search);
});

// Navigate function
function navigate(path, query = {}) {
    const queryString = Object.keys(query).length > 0 
        ? '?' + new URLSearchParams(query).toString() 
        : '';
    window.history.pushState({}, '', path + queryString);
    currentPath.value = path;
    currentQuery.value = new URLSearchParams(queryString);
}

// Route matching
function matchRoute(pattern, path) {
    const patternParts = pattern.split('/').filter(Boolean);
    const pathParts = path.split('/').filter(Boolean);
    
    if (patternParts.length !== pathParts.length) return null;
    
    const params = {};
    for (let i = 0; i < patternParts.length; i++) {
        if (patternParts[i].startsWith(':')) {
            params[patternParts[i].slice(1)] = pathParts[i];
        } else if (patternParts[i] === '*') {
            params._ = pathParts.slice(i).join('/');
            break;
        } else if (patternParts[i] !== pathParts[i]) {
            return null;
        }
    }
    return params;
}

// API client
async function api(endpoint) {
    const response = await fetch('/api' + endpoint);
    if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
    }
    return response.json();
}

async function apiText(endpoint) {
    const response = await fetch('/api' + endpoint);
    if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
    }
    return response.text();
}

// Components

function Header() {
    return html`
        <header>
            <div class="container">
                <h1>📦 Git Server</h1>
                <nav>
                    <a href="/" onClick=${(e) => { e.preventDefault(); navigate('/'); }}>Repositories</a>
                </nav>
            </div>
        </header>
    `;
}

function Home() {
    const [repos, setRepos] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);

    useEffect(() => {
        api('/repos')
            .then(setRepos)
            .catch(e => setError(e.message))
            .finally(() => setLoading(false));
    }, []);

    if (loading) return html`<div class="loading">Loading repositories...</div>`;
    if (error) return html`<div class="error">${error}</div>`;

    return html`
        <div class="container">
            <div class="card">
                <div class="card-header">
                    <h2>Repositories</h2>
                </div>
                ${repos.length === 0 
                    ? html`<div class="empty">No repositories found</div>`
                    : html`
                        <ul class="repo-list">
                            ${repos.map(repo => html`
                                <li key=${repo.name}>
                                    <a href="/repos/${repo.name}" onClick=${(e) => { e.preventDefault(); navigate('/repos/' + repo.name); }}>
                                        ${repo.name}
                                    </a>
                                    <div style="font-size: 12px; color: #57606a; margin-top: 4px;">
                                        ${repo.path}
                                    </div>
                                </li>
                            `)}
                        </ul>
                    `
                }
            </div>
        </div>
    `;
}

function Repository({ name }) {
    const [files, setFiles] = useState([]);
    const [commits, setCommits] = useState([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const [activeTab, setActiveTab] = useState('files');
    const gitRef = currentQuery.value.get('ref') || 'HEAD';
    const path = currentQuery.value.get('path') || '';

    useEffect(() => {
        setLoading(true);
        const queryParams = new URLSearchParams();
        if (gitRef !== 'HEAD') queryParams.set('ref', gitRef);
        if (path) queryParams.set('path', path);
        const query = queryParams.toString() ? '?' + queryParams.toString() : '';
        
        Promise.all([
            api('/repos/' + name + '/tree' + query),
            api('/repos/' + name + '/commits')
        ])
            .then(([filesData, commitsData]) => {
                setFiles(filesData);
                setCommits(commitsData);
            })
            .catch(e => setError(e.message))
            .finally(() => setLoading(false));
    }, [name, gitRef, path]);

    const handleNavToCommit = (hash) => {
        navigate('/repos/' + name, { ref: hash });
    };

    const handleNavToPath = (filePath, type) => {
        if (type === 'dir') {
            navigate('/repos/' + name, { ref: gitRef, path: filePath });
        } else {
            navigate('/repos/' + name + '/blob/' + filePath, { ref: gitRef });
        }
    };

    const handleGoUp = () => {
        const parts = path.split('/');
        parts.pop();
        const newPath = parts.join('/');
        navigate('/repos/' + name, { ref: gitRef, path: newPath || undefined });
    };

    if (loading) return html`<div class="loading">Loading repository...</div>`;
    if (error) return html`<div class="error">${error}</div>`;

    return html`
        <div class="container">
            <div class="card">
                <div class="card-header">
                    <h2>
                        <a href="/" onClick=${(e) => { e.preventDefault(); navigate('/'); }} style="color: #0969da;">Repositories</a>
                        ${' / '}
                        <span>${name}</span>
                        ${gitRef !== 'HEAD' && html`
                            <span style="font-size: 12px; background: #ddf4ff; color: #0969da; padding: 2px 8px; border-radius: 12px; margin-left: 8px;">
                                ${gitRef.substring(0, 7)}
                            </span>
                        `}
                    </h2>
                </div>
                <div class="tabs">
                    <a 
                        href="#" 
                        class=${activeTab === 'files' ? 'active' : ''} 
                        onClick=${(e) => { e.preventDefault(); setActiveTab('files'); }}
                    >
                        📁 Files
                    </a>
                    <a 
                        href="#" 
                        class=${activeTab === 'commits' ? 'active' : ''} 
                        onClick=${(e) => { e.preventDefault(); setActiveTab('commits'); }}
                    >
                        📝 Commits
                    </a>
                </div>
                
                ${activeTab === 'files' && html`
                    ${path && html`
                        <div class="breadcrumb">
                            <a href="#" onClick=${(e) => { e.preventDefault(); navigate('/repos/' + name, { ref: gitRef !== 'HEAD' ? gitRef : undefined }); }}>
                                ${name}
                            </a>
                            ${path.split('/').map((part, i, arr) => {
                                const partPath = arr.slice(0, i + 1).join('/');
                                return html`
                                    ${' / '}
                                    <a href="#" onClick=${(e) => { e.preventDefault(); navigate('/repos/' + name, { ref: gitRef, path: partPath }); }}>
                                        ${part}
                                    </a>
                                `;
                            })}
                        </div>
                    `}
                    ${files.length === 0 
                        ? html`<div class="empty">No files in this ${path ? 'directory' : 'repository'}</div>`
                        : html`
                            <ul class="file-list">
                                ${path && html`
                                    <li>
                                        <span class="icon">📁</span>
                                        <a href="#" onClick=${(e) => { e.preventDefault(); handleGoUp(); }}>..</a>
                                    </li>
                                `}
                                ${files.map(file => html`
                                    <li key=${file.path}>
                                        <span class="icon">${file.type === 'dir' ? '📁' : '📄'}</span>
                                        <a href="#" onClick=${(e) => { e.preventDefault(); handleNavToPath(file.path, file.type); }}>
                                            ${file.name}
                                        </a>
                                        ${file.size !== null && file.size !== undefined && html`
                                            <span style="margin-left: auto; font-size: 12px; color: #57606a;">
                                                ${formatSize(file.size)}
                                            </span>
                                        `}
                                    </li>
                                `)}
                            </ul>
                        `
                    }
                `}
                
                ${activeTab === 'commits' && html`
                    ${commits.length === 0 
                        ? html`<div class="empty">No commits yet</div>`
                        : html`
                            <ul class="commit-list">
                                ${commits.map(commit => html`
                                    <li key=${commit.hash}>
                                        <div>
                                            <span 
                                                class="commit-hash" 
                                                onClick=${() => handleNavToCommit(commit.hash)}
                                                title="Browse files at this commit"
                                            >
                                                ${commit.short_hash}
                                            </span>
                                            <span class="commit-message" style="margin-left: 8px;">
                                                ${commit.message}
                                            </span>
                                        </div>
                                        <div class="commit-meta">
                                            ${commit.author} committed on ${formatDate(commit.date)}
                                        </div>
                                    </li>
                                `)}
                            </ul>
                        `
                    }
                `}
            </div>
        </div>
    `;
}

function BlobView({ name, path }) {
    const [content, setContent] = useState('');
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(null);
    const gitRef = currentQuery.value.get('ref') || 'HEAD';

    useEffect(() => {
        setLoading(true);
        const queryParams = new URLSearchParams();
        if (gitRef !== 'HEAD') queryParams.set('ref', gitRef);
        const query = queryParams.toString() ? '?' + queryParams.toString() : '';
        
        apiText('/repos/' + name + '/blob/' + path + query)
            .then(setContent)
            .catch(e => setError(e.message))
            .finally(() => setLoading(false));
    }, [name, path, gitRef]);

    const handleBack = () => {
        const parts = path.split('/');
        parts.pop();
        const parentPath = parts.join('/');
        navigate('/repos/' + name, { ref: gitRef !== 'HEAD' ? gitRef : undefined, path: parentPath || undefined });
    };

    if (loading) return html`<div class="loading">Loading file...</div>`;
    if (error) return html`<div class="error">${error}</div>`;

    return html`
        <div class="container">
            <div class="card">
                <div class="card-header">
                    <h2>
                        <a href="/" onClick=${(e) => { e.preventDefault(); navigate('/'); }} style="color: #0969da;">Repositories</a>
                        ${' / '}
                        <a href="#" onClick=${(e) => { e.preventDefault(); navigate('/repos/' + name, { ref: gitRef !== 'HEAD' ? gitRef : undefined }); }} style="color: #0969da;">${name}</a>
                        ${' / '}
                        ${path.split('/').map((part, i, arr) => {
                            const isLast = i === arr.length - 1;
                            const partPath = arr.slice(0, i + 1).join('/');
                            if (isLast) {
                                return html`<span>${part}</span>`;
                            }
                            return html`
                                <a href="#" onClick=${(e) => { e.preventDefault(); navigate('/repos/' + name, { ref: gitRef, path: partPath }); }} style="color: #0969da;">
                                    ${part}
                                </a>
                                ${' / '}
                            `;
                        })}
                        ${gitRef !== 'HEAD' && html`
                            <span style="font-size: 12px; background: #ddf4ff; color: #0969da; padding: 2px 8px; border-radius: 12px; margin-left: 8px;">
                                ${gitRef.substring(0, 7)}
                            </span>
                        `}
                    </h2>
                </div>
                <pre class="blob-content">${content}</pre>
            </div>
        </div>
    `;
}

function formatSize(bytes) {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
}

function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
    });
}

function NotFound() {
    return html`
        <div class="container">
            <div class="card">
                <div class="card-body">
                    <h2>404 - Page Not Found</h2>
                    <p style="margin-top: 16px;">
                        <a href="/" onClick=${(e) => { e.preventDefault(); navigate('/'); }}>Go back home</a>
                    </p>
                </div>
            </div>
        </div>
    `;
}

function App() {
    // Use effect to trigger re-render on path changes
    const [, setRender] = useState(0);
    
    useEffect(() => {
        return effect(() => {
            // Subscribe to path changes
            currentPath.value;
            currentQuery.value;
            setRender(r => r + 1);
        });
    }, []);

    const path = currentPath.value;
    
    // Route matching
    let content;
    let params;
    
    if (path === '/') {
        content = html`<${Home} />`;
    } else if (params = matchRoute('/repos/:name', path)) {
        content = html`<${Repository} name=${params.name} />`;
    } else if (params = matchRoute('/repos/:name/blob/*', path)) {
        // Extract the blob path from the URL
        const blobPath = path.replace('/repos/' + params.name + '/blob/', '');
        content = html`<${BlobView} name=${params.name} path=${blobPath} />`;
    } else {
        content = html`<${NotFound} />`;
    }

    return html`
        <${Fragment}>
            <${Header} />
            ${content}
        <//>
    `;
}

// Render the app
render(html`<${App} />`, document.getElementById('app'));
