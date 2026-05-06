(function (global) {
    'use strict';

    var DEFAULT_ARRAY_PAGE = 50;

    function typeOf(value) {
        if (value === null) return 'null';
        if (Array.isArray(value)) return 'array';
        return typeof value;
    }

    function escapeText(str) {
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    function formatPathSegment(key, isArrayIndex) {
        if (isArrayIndex) return '[' + key + ']';
        if (/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key)) return '.' + key;
        return '[' + JSON.stringify(key) + ']';
    }

    function joinPath(parentPath, key, isArrayIndex) {
        if (parentPath === '' && !isArrayIndex) {
            return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(key) ? key : '[' + JSON.stringify(key) + ']';
        }
        return parentPath + formatPathSegment(key, isArrayIndex);
    }

    function makeBtn(cls, label, title) {
        var b = document.createElement('button');
        b.type = 'button';
        b.className = cls;
        if (label !== undefined) b.textContent = label;
        if (title) b.title = title;
        return b;
    }

    function copyToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            return navigator.clipboard.writeText(text);
        }
        return new Promise(function (resolve, reject) {
            try {
                var ta = document.createElement('textarea');
                ta.value = text;
                ta.setAttribute('readonly', '');
                ta.style.position = 'absolute';
                ta.style.left = '-9999px';
                document.body.appendChild(ta);
                ta.select();
                document.execCommand('copy');
                document.body.removeChild(ta);
                resolve();
            } catch (e) {
                reject(e);
            }
        });
    }

    function flashCopied(btn) {
        btn.classList.add('is-copied');
        var prev = btn.textContent;
        btn.textContent = '✓';
        setTimeout(function () {
            btn.classList.remove('is-copied');
            btn.textContent = prev;
        }, 900);
    }

    function buildPathCopy(path) {
        if (!path) return null;
        var b = makeBtn('json-tree__path-copy', '⧉', 'Copy path: ' + path);
        b.setAttribute('aria-label', 'Copy path ' + path);
        b.dataset.path = path;
        b.addEventListener('click', function (e) {
            e.stopPropagation();
            copyToClipboard(path).then(function () {
                flashCopied(b);
            }).catch(function () {});
        });
        return b;
    }

    function valueClass(t) {
        if (t === 'boolean') return 'bool';
        if (t === 'undefined') return 'null';
        return t;
    }

    function renderPrimitive(value) {
        var span = document.createElement('span');
        var t = typeOf(value);
        span.classList.add('json-tree__value', 'json-tree__value--' + valueClass(t));
        if (t === 'string') {
            span.textContent = '"' + value + '"';
        } else if (t === 'null') {
            span.textContent = 'null';
        } else if (t === 'undefined') {
            span.textContent = 'undefined';
        } else {
            span.textContent = String(value);
        }
        return span;
    }

    function summaryText(value) {
        var t = typeOf(value);
        if (t === 'array') return 'Array(' + value.length + ')';
        if (t === 'object') {
            var keys = Object.keys(value);
            return '{' + keys.length + (keys.length === 1 ? ' key' : ' keys') + '}';
        }
        return '';
    }

    function renderNode(opts) {
        var value = opts.value;
        var keyLabel = opts.keyLabel; // string or null
        var isArrayIndex = !!opts.isArrayIndex;
        var path = opts.path;
        var depth = opts.depth;
        var seen = opts.seen;
        var startCollapsed = !!opts.startCollapsed;

        var node = document.createElement('div');
        node.className = 'json-tree__node';
        node.style.setProperty('--depth', String(depth));

        var row = document.createElement('span');
        row.className = 'json-tree__row';
        node.appendChild(row);

        var t = typeOf(value);
        var isContainer = (t === 'object' || t === 'array');

        var toggle = makeBtn('json-tree__toggle', '', isContainer ? 'Toggle' : '');
        if (!isContainer) toggle.classList.add('json-tree__toggle--leaf');
        row.appendChild(toggle);

        if (keyLabel !== null && keyLabel !== undefined) {
            var keyEl = document.createElement('span');
            keyEl.className = isArrayIndex ? 'json-tree__index' : 'json-tree__key';
            keyEl.textContent = keyLabel;
            row.appendChild(keyEl);
        }

        if (isContainer) {
            // Circular reference defense
            if (seen.indexOf(value) !== -1) {
                var circ = document.createElement('span');
                circ.className = 'json-tree__circular';
                circ.textContent = '[circular]';
                row.appendChild(circ);
                return node;
            }

            var openCh = (t === 'array') ? '[' : '{';
            var closeCh = (t === 'array') ? ']' : '}';

            var open = document.createElement('span');
            open.className = 'json-tree__punct';
            open.textContent = openCh;
            row.appendChild(open);

            var summary = document.createElement('span');
            summary.className = 'json-tree__summary';
            summary.textContent = summaryText(value);
            row.appendChild(summary);

            var close = document.createElement('span');
            close.className = 'json-tree__punct';
            close.textContent = closeCh;
            close.style.marginLeft = '0.25rem';
            row.appendChild(close);

            if (path) {
                var pc = buildPathCopy(path);
                if (pc) row.appendChild(pc);
            }

            var children = document.createElement('div');
            children.className = 'json-tree__children';
            node.appendChild(children);

            var childrenBuilt = false;
            var renderedCount = 0;

            function buildChildren(limit) {
                var nextSeen = seen.concat([value]);
                var keys = (t === 'array') ? null : Object.keys(value);
                var total = (t === 'array') ? value.length : keys.length;
                var stop = Math.min(total, limit);

                for (var i = renderedCount; i < stop; i++) {
                    var k, v, label, isIdx;
                    if (t === 'array') {
                        k = i;
                        v = value[i];
                        label = String(i);
                        isIdx = true;
                    } else {
                        k = keys[i];
                        v = value[k];
                        label = k;
                        isIdx = false;
                    }
                    var childPath = joinPath(path, isIdx ? k : k, isIdx);
                    var childNode = renderNode({
                        value: v,
                        keyLabel: label,
                        isArrayIndex: isIdx,
                        path: childPath,
                        depth: depth + 1,
                        seen: nextSeen,
                        startCollapsed: false
                    });
                    children.appendChild(childNode);
                }
                renderedCount = stop;

                // existing "expand more" button
                var existing = children.querySelector(':scope > .json-tree__expand-more');
                if (existing) existing.remove();

                if (renderedCount < total) {
                    var more = makeBtn('json-tree__expand-more', 'Show ' + Math.min(DEFAULT_ARRAY_PAGE, total - renderedCount) + ' more (' + (total - renderedCount) + ' remaining)');
                    more.style.setProperty('--depth', String(depth));
                    more.addEventListener('click', function (e) {
                        e.stopPropagation();
                        buildChildren(renderedCount + DEFAULT_ARRAY_PAGE);
                    });
                    children.appendChild(more);
                }
            }

            function ensureBuilt() {
                if (childrenBuilt) return;
                childrenBuilt = true;
                var total = (t === 'array') ? value.length : Object.keys(value).length;
                var initial = (t === 'array' && total > DEFAULT_ARRAY_PAGE) ? DEFAULT_ARRAY_PAGE : total;
                buildChildren(initial);
            }

            // Decide initial collapsed state. Large arrays start collapsed by default.
            var totalChildren = (t === 'array') ? value.length : Object.keys(value).length;
            var collapsed = startCollapsed || (t === 'array' && totalChildren > DEFAULT_ARRAY_PAGE) || depth >= 6;

            function setCollapsed(c) {
                if (c) {
                    node.classList.add('is-collapsed');
                    children.style.display = 'none';
                } else {
                    node.classList.remove('is-collapsed');
                    children.style.display = '';
                    ensureBuilt();
                }
            }

            setCollapsed(collapsed);

            toggle.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });

            // also let clicking the punctuation toggle
            open.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });
            close.addEventListener('click', function (e) {
                e.stopPropagation();
                setCollapsed(!node.classList.contains('is-collapsed'));
            });
        } else {
            row.appendChild(renderPrimitive(value));
            if (path) {
                var pc2 = buildPathCopy(path);
                if (pc2) row.appendChild(pc2);
            }
        }

        return node;
    }

    function mountJsonTree(rootEl, value, options) {
        if (!rootEl) return;
        options = options || {};
        rootEl.innerHTML = '';
        rootEl.classList.add('json-tree');

        var rootLabel = options.rootLabel || rootEl.dataset.rootLabel || null;
        var startCollapsed = options.startCollapsed === true || rootEl.dataset.collapsed === '1';
        var rootPath = rootLabel || '';

        try {
            var node = renderNode({
                value: value,
                keyLabel: rootLabel,
                isArrayIndex: false,
                path: rootPath,
                depth: 0,
                seen: [],
                startCollapsed: startCollapsed
            });
            rootEl.appendChild(node);
        } catch (e) {
            var err = document.createElement('div');
            err.className = 'json-tree__error';
            err.textContent = 'Failed to render JSON: ' + (e && e.message ? e.message : String(e));
            rootEl.appendChild(err);
        }
    }

    function parseDataAttr(raw) {
        if (raw === undefined || raw === null) return undefined;
        var trimmed = String(raw).trim();
        if (trimmed === '') return undefined;
        try {
            return JSON.parse(trimmed);
        } catch (e) {
            return { __parseError: e.message, raw: trimmed };
        }
    }

    function autoMount(root) {
        var scope = root || document;
        var els = scope.querySelectorAll('.json-tree[data-json]');
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (el.dataset.jsonTreeMounted === '1') continue;
            el.dataset.jsonTreeMounted = '1';
            var value = parseDataAttr(el.getAttribute('data-json'));
            if (value && typeof value === 'object' && value.__parseError) {
                el.innerHTML = '';
                var err = document.createElement('div');
                err.className = 'json-tree__error';
                err.textContent = 'Invalid JSON: ' + value.__parseError;
                el.appendChild(err);
                continue;
            }
            mountJsonTree(el, value);
        }
    }

    var ns = global.SystempromptAdmin = global.SystempromptAdmin || {};
    ns.jsonTree = {
        mount: mountJsonTree,
        mountJsonTree: mountJsonTree,
        autoMount: autoMount
    };

    // Back-compat: also expose via AdminApp namespace used elsewhere.
    if (global.AdminApp) {
        global.AdminApp.JsonTree = ns.jsonTree;
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () { autoMount(); });
    } else {
        autoMount();
    }
})(window);
