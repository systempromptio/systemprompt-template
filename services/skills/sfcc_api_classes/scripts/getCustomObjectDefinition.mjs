// @ts-check
import fs from 'node:fs/promises';
import { findAllTags, getLineOfOffset } from './xmlHelper.mjs';

// node getCustomObjectDefinition.mjs <path-to-xml-file> <CustomObjectType>

const [,, xmlFilePath, customObjectType] = process.argv;

if (!xmlFilePath || !customObjectType) {
  console.error('Usage: node getCustomObjectDefinition.mjs <path-to-xml-file> <CustomObjectType>');
  process.exit(1);
}

const customObjectTypeLower = customObjectType.toLowerCase();

const xmlContent = await fs.readFile(xmlFilePath, 'utf-8');

const metadataTag = findAllTags('metadata', xmlContent)[0];

if (!metadataTag) {
    console.error('No <metadata> tag found in the XML file.');
    process.exit(1);
}


const typeExtensionTags = findAllTags('custom-type', xmlContent, metadataTag);

for (const typeExtensionTag of typeExtensionTags) {

    if (typeExtensionTag) {

        if (!typeExtensionTag.attrs['type-id'] || typeExtensionTag.attrs['type-id'].toLowerCase() !== customObjectTypeLower) {
            continue;
        }

        console.log(`custom-type: Range #${getLineOfOffset(typeExtensionTag.startIndex, xmlContent)}-${getLineOfOffset(typeExtensionTag.endIndex, xmlContent)}`);

        const attributeDefinitionsTags = findAllTags('attribute-definition', xmlContent, typeExtensionTag);

        if (attributeDefinitionsTags.length) {
            console.log('Short version:');

            for (const attributeDefinitionTag of attributeDefinitionsTags) {
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

console.log(`Custom Object Type "${customObjectTypeLower}" not found.`);
process.exit(1);
