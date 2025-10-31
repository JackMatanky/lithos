#!/usr/bin/env node

/**
 * JSON Schema Validation Script for Lithos Domain Models
 *
 * Usage:
 *   node scripts/validate-schema.js <data-file>
 *   node scripts/validate-schema.js schemas/examples/note-example.json
 */

const fs = require('fs');
const path = require('path');
const Ajv = require('ajv').default;

// Load the main schema
const schemaPath = path.join(__dirname, '..', 'schemas', 'lithos-domain-schema.json');
const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));

// Initialize AJV validator
const ajv = new Ajv({
  strict: false,  // Disable strict mode for compatibility
  allErrors: true // Show all validation errors
});

const validate = ajv.compile(schema);

// Get data file from command line arguments
const dataFile = process.argv[2];

if (!dataFile) {
  console.log('Usage: node scripts/validate-schema.js <data-file>');
  console.log('Example: node scripts/validate-schema.js schemas/examples/note-example.json');
  process.exit(1);
}

// Check if data file exists
if (!fs.existsSync(dataFile)) {
  console.error(`Error: File not found: ${dataFile}`);
  process.exit(1);
}

try {
  // Load and parse the data file
  const data = JSON.parse(fs.readFileSync(dataFile, 'utf8'));

  // Validate against schema
  const valid = validate(data);

  if (valid) {
    console.log(`✅ ${dataFile} is valid according to the Lithos domain schema`);

    // Show what type of data was validated
    if (data.id && data.frontmatter) {
      console.log(`   📄 Note: ${data.id}`);
    } else if (data.name && data.properties) {
      console.log(`   📋 Schema: ${data.name}`);
    } else if (data.properties && typeof data.properties === 'object') {
      console.log(`   🏦 PropertyBank with ${Object.keys(data.properties).length} properties`);
    } else {
      console.log(`   📦 Valid data structure`);
    }
  } else {
    console.log(`❌ ${dataFile} is invalid:`);
    validate.errors.forEach(error => {
      console.log(`   • ${error.instancePath || 'root'}: ${error.message}`);
      if (error.data !== undefined) {
        console.log(`     Value: ${JSON.stringify(error.data)}`);
      }
    });
    process.exit(1);
  }
} catch (error) {
  console.error(`Error processing ${dataFile}:`, error.message);
  process.exit(1);
}
