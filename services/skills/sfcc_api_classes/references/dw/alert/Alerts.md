# dw.alert.Alerts

## Overview
Allows creation, removal, re-validation, and retrieval of alerts visible to Business Manager users. Alerts are registered via an `alerts.json` descriptor in a cartridge and referenced by ID throughout the API.

## Description
Alerts must be defined in a cartridge's `alerts.json` and referenced by their ID. Menu actions for alerts are found in the `bm_extensions.xml` file of a Business Manager extension cartridge.

```ts
declare class Alerts {
    /**
     * Creates a new alert for the given ID. If such an alert already exists, no new one is created.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param params Parameters which may be shown in the alert message.
     */
    static addAlert(alertDescriptorID: String, ...params: String[]): void;

    /**
     * Creates a new alert for the given ID and context object. Multiple alerts for the same ID may exist if referencing different objects.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObject The context object.
     * @param params Parameters which may be shown in the alert message.
     */
    static addAlert(alertDescriptorID: String, contextObject: PersistentObject, ...params: String[]): void;

    /**
     * Creates a new alert for the given ID and context object ID. Multiple alerts for the same ID may exist if referencing different objects.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObjectID The ID of the referenced object.
     * @param params Parameters which may be shown in the alert message.
     */
    static addAlert(alertDescriptorID: String, contextObjectID: String, ...params: String[]): void;

    /**
     * Retrieves all alerts for a set of alert descriptor IDs.
     * @param alertDescriptorIDs The IDs of the referenced alert descriptions.
     * @returns The list of alerts (of type Alert).
     */
    static getAlerts(...alertDescriptorIDs: String[]): List;

    /**
     * Retrieves all alerts for a set of alert descriptor IDs and the given context object ID.
     * @param contextObjectID The ID of the referenced object.
     * @param alertDescriptorIDs The IDs of the referenced alert descriptions.
     * @returns The list of alerts (of type Alert).
     */
    static getAlertsForContextObject(contextObjectID: String, ...alertDescriptorIDs: String[]): List;

    /**
     * Retrieves all alerts for a set of alert descriptor IDs and the given context object.
     * @param contextObject The context object.
     * @param alertDescriptorIDs The IDs of the referenced alert descriptions.
     * @returns The list of alerts (of type Alert).
     */
    static getAlertsForContextObject(contextObject: PersistentObject, ...alertDescriptorIDs: String[]): List;

    /**
     * Removes all alerts for the given alert descriptor ID.
     * @param alertDescriptorID The ID of the referenced alert description.
     */
    static removeAlert(alertDescriptorID: String): void;

    /**
     * Removes the alert for the given alert description and context object.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObject The context object.
     */
    static removeAlert(alertDescriptorID: String, contextObject: PersistentObject): void;

    /**
     * Removes the alert for the given alert description and context object ID.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObjectID The context object ID.
     */
    static removeAlert(alertDescriptorID: String, contextObjectID: String): void;

    /**
     * Re-evaluates the process function, and creates or removes the respective alert.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param processFunction The validation function. Must return true when the alert needs to be created.
     * @param params Parameters which may be shown in the alert message.
     */
    static revalidateAlert(alertDescriptorID: String, processFunction: Function, ...params: String[]): void;

    /**
     * Re-evaluates the process function, and creates or removes the respective alert for the given context object.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObject The context object for which the validation is done.
     * @param processFunction The validation function. Must return true when the alert needs to be created.
     * @param params Parameters which may be shown in the alert message.
     */
    static revalidateAlert(alertDescriptorID: String, contextObject: PersistentObject, processFunction: Function, ...params: String[]): void;

    /**
     * Re-evaluates the process function, and creates or removes the respective alert for the given context object and ID.
     * @param alertDescriptorID The ID of the referenced alert description.
     * @param contextObject The context object for which the validation is done.
     * @param contextObjectID The ID of the context object.
     * @param processFunction The validation function. Must return true when the alert needs to be created.
     * @param params Parameters which may be shown in the alert message.
     */
    static revalidateAlert(alertDescriptorID: String, contextObject: Object, contextObjectID: String, processFunction: Function, ...params: String[]): void;
}
```
