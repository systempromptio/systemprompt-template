import fs from 'node:fs/promises';

const stepName = process.argv?.[2]?.toLowerCase() || null;

const steptypesFiles = fs.glob('**/steptypes.json', { exclude: ['**/node_modules/**', '**/cartridge/**'] });

for await (const file of  steptypesFiles) {
    const fileContent = await fs.readFile(file, 'utf8');
    const jsonContent = JSON.parse(fileContent);

    let scriptModuleSteps = jsonContent?.['step-types']?.['script-module-step'] ?? [];

    if (stepName) {
        scriptModuleSteps = scriptModuleSteps.filter(step => step['@type-id']?.toLowerCase().includes(stepName));
    }

    //chunk-script-module-step
    let chunkScriptModuleSteps = jsonContent?.['step-types']?.['chunk-script-module-step'] ?? [];

    if (stepName) {
        chunkScriptModuleSteps = chunkScriptModuleSteps.filter(step => step['@type-id']?.toLowerCase().includes(stepName));
    }

    if (scriptModuleSteps.length || chunkScriptModuleSteps.length) {
        console.log(`- ${file}`);
    }

    if (scriptModuleSteps.length) {
        console.log('  - script-module-steps:');

        for (const step of scriptModuleSteps) {
            console.log(`    - [${getLineOfText(step['@type-id'], fileContent)}]${step.description ? ` (${step.description})`: ''} ${step['@type-id']} -> ${step['module']}`);
        }
    }
    if (chunkScriptModuleSteps.length) {
        console.log('  - chunk-script-module-steps:');

        for (const chunke of chunkScriptModuleSteps) {
            console.log(`    - [${getLineOfText(chunke['@type-id'], fileContent)}]${chunke.description ? ` (${chunke.description})`: ''} ${chunke['@type-id']} -> ${chunke['module']}`);
        }
    }
}

console.log('\n\n[n] - contains line number in the file where job is defined\n');

/**
 * 
 * @param {string} text 
 * @param {string} content 
 */
function getLineOfText(text, content) {
    const maxLength = content.length
    const offset = content.indexOf(text);
    if (offset === -1) return -1;
    let line = 1;
    for (let i = 0; i < offset && i < maxLength; i++) {
        if (content[i] === '\n') line++;
    }
    return line;
}