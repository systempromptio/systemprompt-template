// @ts-check
import fs from 'node:fs/promises';
import { findAllTags, getLineOfOffset } from './xmlHelper.mjs';

// node getCustomAttributeDefinition.mjs <path-to-xml-file> <CustomAttributeName>

const [,, xmlFilePath, customAttributeName] = process.argv;

if (!xmlFilePath || !customAttributeName) {
  console.error('Usage: node getCustomAttributeDefinition.mjs <path-to-xml-file> <CustomAttributeName>');
  process.exit(1);
}

const customAttributeNameLower = customAttributeName.toLowerCase();

const xmlContent = await fs.readFile(xmlFilePath, 'utf-8');

const metadataTag = findAllTags('metadata', xmlContent)[0];

if (!metadataTag) {
    console.error('No <metadata> tag found in the XML file.');
    process.exit(1);
}


const typeExtensionTags = findAllTags('type-extension', xmlContent, metadataTag);

for (const typeExtensionTag of typeExtensionTags) {

    if (typeExtensionTag) {

        if (!typeExtensionTag.attrs['type-id'] || typeExtensionTag.attrs['type-id'].toLowerCase() !== customAttributeNameLower) {
            continue;
        }

        console.log(`type-extension: Range #${getLineOfOffset(typeExtensionTag.startIndex, xmlContent)}-${getLineOfOffset(typeExtensionTag.endIndex, xmlContent)}`);

        const customAttributeDefinitionsTag = findAllTags('custom-attribute-definitions', xmlContent, typeExtensionTag)[0];

        if (customAttributeDefinitionsTag) {
            console.log(`custom-attribute-definitions: Range #${getLineOfOffset(customAttributeDefinitionsTag.startIndex, xmlContent)}-${getLineOfOffset(customAttributeDefinitionsTag.endIndex, xmlContent)}`);
            console.log('Short version:');

            const attributeDefinitionTags = findAllTags('attribute-definition', xmlContent, customAttributeDefinitionsTag);

            for (const attributeDefinitionTag of attributeDefinitionTags) {
                const attrName = attributeDefinitionTag.attrs['attribute-id'];

                const displayNameTag = findAllTags('display-name', xmlContent, attributeDefinitionTag);
                const displayNameRaw = xmlContent.substring(displayNameTag[0].startIndex, displayNameTag[0].endIndex) ?? '';
                const displayName = displayNameRaw.substring(displayNameRaw.indexOf('>') + 1)

                console.log(` - ${attrName}#${getLineOfOffset(attributeDefinitionTag.startIndex, xmlContent)}-${getLineOfOffset(attributeDefinitionTag.endIndex, xmlContent)}${displayName.length > 0 ? ` ${displayName}` : ''}`);
            }

            console.log('\nDetailed information may be retrieved from the XML file directly via provided line numbers.');
        }

        process.exit(0);
    }
};

console.log(`Custom Attribute "${customAttributeName}" not found.`);
process.exit(1);
