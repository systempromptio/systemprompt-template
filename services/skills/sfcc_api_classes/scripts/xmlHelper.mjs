


/**
 * 
 * @param {number} offset 
 * @param {string} content 
 * @returns 
 */
export function getLineOfOffset(offset, content) {
    if (offset === -1) return -1;
    let line = 1;
    let pos = 0;
    while (pos < offset && pos !== -1) {
        pos = content.indexOf('\n', pos);
        if (pos !== -1 && pos < offset) {
            line++;
            pos++;
        }
    }
    return line;
}

/**
 * @param {string} tagName 
 * @param {string} content 
 * @param {number} startFrom 
 * @param {number} endAt 
 */
function findTagContent(tagName, content, startFrom = 0, endAt = content.length) {
    const startTag = `<${tagName} `;
    const endTag = `</${tagName}>`;

    const startIndex = content.indexOf(startTag, startFrom);
    if (startIndex === -1) return null;

    const opendEndIndex = findTagEnd(content, startIndex + startTag.length);
    if (opendEndIndex === -1) return null;
    if (opendEndIndex > endAt) return null;

    const endChar = content.charAt(opendEndIndex);

    if (endChar === '/') {
        const attrs = parseAttributes(content.substring(startIndex + startTag.length, opendEndIndex - 1).trim());
        return {
            attrs,
            startIndex: startIndex,
            endIndex: opendEndIndex + 2
        }
    }

    const endIndex = content.indexOf(endTag, startIndex);
    if (endIndex === -1) return null;
    if (endIndex > endAt) return null;

    const attrs = parseAttributes(content.substring(startIndex + startTag.length, opendEndIndex).trim());

    return {
        attrs,
        startIndex, 
        endIndex
    }
}

/**
* @param {string} tagName 
* @param {string} content 
* @param {NonNullable<ReturnType<typeof findTagContent>>} [inTag] 

*/
export function findAllTags(tagName, content, inTag) {
    const result = [];
    const endAt = inTag?.endIndex ?? content.length;
    let possition = inTag?.startIndex ?? 0;

    while (possition < endAt) {
        const tag = findTagContent(tagName, content, possition, endAt);
        if (!tag) {
            break;
        }
        result.push(tag);
        possition = tag.endIndex;
    }

    return result;
}

/**
 * 
 * @param {string} attrString 
 * @returns 
 */
function parseAttributes(attrString) {
    /**
     * @type {Record<string, string>}
     */
    const attributes = {};
    const attrArr = (attrString || '').split(/\s+/);
    for (const attr of attrArr) {
        const [key, value] = attr.split('=');
        attributes[key] = value?.replace(/"/g, '') ?? '';
    }
    return attributes;
}

/**
 * @param {string} content 
 * @param {number} startFrom 
 */
function findTagEnd(content, startFrom) {
    const length = content.length;
    let possition = startFrom;
    let inQuotes = false;

    while (possition < length) {
        const char = content.charAt(possition);
        if (char === '"') {
            inQuotes = !inQuotes;
        }
        if (inQuotes) {
            possition++;
            continue;
        }
        if (char === '>') {
            return possition;
        } else if (char === '<') {
            return -1;
        } else if (char === '/') {
            if (content.charAt(possition + 1) === '>') {
                return possition + 1;
            } else {
                return -1;
            }
        }
        possition++;
    }

    return -1;
}
