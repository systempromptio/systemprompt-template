# dw.object.ObjectAttributeDefinition

## Overview
Represents the definition of an object's attribute.

## Description
Defines metadata for a single object attribute: its ID, display name, value type, allowed values, default, unit, and whether it's mandatory, system-defined, or multi-valued.

```ts
declare class ObjectAttributeDefinition  {
    /** Boolean value type. */
    static VALUE_TYPE_BOOLEAN: 8

    /** Date value type. */
    static VALUE_TYPE_DATE: 6

    /** Date and Time value type. */
    static VALUE_TYPE_DATETIME: 11

    /** Email value type. */
    static VALUE_TYPE_EMAIL: 12

    /** Enum of int value type. */
    static VALUE_TYPE_ENUM_OF_INT: 31

    /** Enum of String value type. */
    static VALUE_TYPE_ENUM_OF_STRING: 33

    /** HTML value type. */
    static VALUE_TYPE_HTML: 5

    /** Image value type. */
    static VALUE_TYPE_IMAGE: 7

    /** int value type. */
    static VALUE_TYPE_INT: 1

    /** Money value type. */
    static VALUE_TYPE_MONEY: 9

    /** Number value type. */
    static VALUE_TYPE_NUMBER: 2

    /** Password value type. */
    static VALUE_TYPE_PASSWORD: 13

    /** Quantity value type. */
    static VALUE_TYPE_QUANTITY: 10

    /** Set of int value type. */
    static VALUE_TYPE_SET_OF_INT: 21

    /** Set of Number value type. */
    static VALUE_TYPE_SET_OF_NUMBER: 22

    /** Set of String value type. */
    static VALUE_TYPE_SET_OF_STRING: 23

    /** String value type. */
    static VALUE_TYPE_STRING: 3

    /** Text value type. */
    static VALUE_TYPE_TEXT: 4

    /** All attribute groups the attribute is assigned to. */
    attributeGroups: dw.util.Collection // (Read Only)

    /** Default value definition or null. */
    defaultValue: dw.object.ObjectAttributeValueDefinition // (Read Only)

    /** Display name used in UI. */
    displayName: string // (Read Only)

    /** The ID of the attribute definition. */
    ID: string // (Read Only)

    /** True if attribute is primary key. */
    key: boolean // (Read Only)

    /** True if attribute is mandatory. */
    mandatory: boolean // (Read Only)

    /** True if attribute supports multiple values. */
    multiValueType: boolean // (Read Only)

    /** Object type definition that owns this attribute. */
    objectTypeDefinition: dw.object.ObjectTypeDefinition // (Read Only)

    /** True if attribute is of 'Set of' type (deprecated). */
    setValueType: boolean // (Read Only)

    /** True if attribute is system-defined. */
    system: boolean // (Read Only)

    /** Unit representation for the attribute (e.g., inches). */
    unit: string // (Read Only)

    /** Collection of allowed values (ObjectAttributeValueDefinition instances). */
    values: dw.util.Collection // (Read Only)

    /** Numeric code for the attribute's value type. */
    valueTypeCode: number // (Read Only)

    /** Returns all attribute groups the attribute is assigned to. */
    getAttributeGroups(): dw.util.Collection

    /** Return the default value for the attribute or null if none is defined. */
    getDefaultValue(): dw.object.ObjectAttributeValueDefinition

    /** Returns the display name for the attribute. */
    getDisplayName(): string

    /** Returns the ID of the attribute definition. */
    getID(): string

    /** Returns the object type definition in which this attribute is defined. */
    getObjectTypeDefinition(): dw.object.ObjectTypeDefinition

    /** Returns the attribute's unit representation. */
    getUnit(): string

    /** Returns the list of attribute values. */
    getValues(): dw.util.Collection

    /** Returns a code for the data type stored in the attribute. */
    getValueTypeCode(): number

    /** Identifies if the attribute represents the primary key of the object. */
    isKey(): boolean

    /** Checks if this attribute is mandatory. */
    isMandatory(): boolean

    /** Returns true if the attribute can have multiple values. */
    isMultiValueType(): boolean

    /** Returns true if the attribute is of type 'Set of'. Deprecated; use isMultiValueType(). */
    isSetValueType(): boolean

    /** Indicates if the attribute is a pre-defined system attribute. */
    isSystem(): boolean

    /** Returns whether values of this attribute should be encoded in ISML templates. */
    requiresEncoding(): boolean
}
```
