# dw.web.FormGroup

## Overview
Central container element for form handling. Groups fields and sub-forms with index or associative array access.

## Description
The class is the central class within the whole form handling. It is the container element for fields and other form elements. A form group can contain other forms, also called sub-forms.

Access to the elements of a form is provided via an index based access or via an associative array access. For example, the field "firstname" can be accessed with the expression "myform.firstname".

## All Known Subclasses
Form, FormList, FormListItem

```
Object
  dw.web.FormElement
    dw.web.FormGroup
```

```ts
declare class FormGroup extends FormElement {
	/**
	 * The number of elements in the form.
	 */
	readonly childCount: number

	/**
	 * A form-wide error message. If no error message is present the method returns null.
	 */
	readonly error: string

	/**
	 * The object that was bound to this form group.
	 */
	readonly object: Object

	/**
	 * The action that was submitted with the last request. The action is set independent whether the form must be valid for this action. Returns null if no action was submitted.
	 */
	readonly submittedAction: FormAction

	/**
	 * The action that was triggered with the last request. An action is only marked as triggered if the constraints regarding form validation are met. Returns null if no action was triggered.
	 */
	readonly triggeredAction: FormAction

	/**
	 * Copies the value from a form into the object which was previously bound to the form. Equivalent to the pipelet AcceptForm.
	 */
	accept(): void

	/**
	 * Updates the form with values from the given object. Equivalent to pipelet UpdateFormWithObject. Also binds the object to the form.
	 * @param obj - The object from which the values are read
	 */
	copyFrom(obj: Object): void

	/**
	 * Updates the object with the values from the form. Equivalent to pipelet UpdateObjectWithForm. Requires a submitted form.
	 * @param obj - The object which is updated from the form
	 */
	copyTo(obj: Object): void

	/**
	 * Returns the number of elements in the form.
	 * @returns The number of elements in the form
	 */
	getChildCount(): number

	/**
	 * Returns a form-wide error message. If no error message is present returns null.
	 * @returns A form-wide error message or null
	 */
	getError(): string

	/**
	 * Returns the object that was bound to this form group.
	 * @returns The bound object
	 */
	getObject(): Object

	/**
	 * Returns the action that was submitted with the last request. Returns null if no action was submitted.
	 * @returns The submitted action or null
	 */
	getSubmittedAction(): FormAction

	/**
	 * Returns the action that was triggered with the last request. An action is only marked as triggered if form validation constraints are met.
	 * @returns The triggered action or null
	 */
	getTriggeredAction(): FormAction
}
```
