 # dw.catalog.ProductOptionModel

 ## Overview
 Represents the option model for a specific product and currency, exposing configured options, their values, prices, and selected value management.

 ## Description
 Provides accessors for product options and option values and allows updating which option values are selected. URLs for selecting option values can be constructed.

 ```ts
 declare class ProductOptionModel  {
    /** The collection of product options. */
    readonly options: Collection

    /** Returns the product option for the specified ID. */
    getOption(optionID: string): ProductOption

    /** Returns the collection of product options. */
    getOptions(): Collection

    /**
     * Returns the product option value for the passed value id in the context of the passed option.
     * @param option - The option to get the specified value for.
     * @param valueID - The id of the value to retrieve.
     */
    getOptionValue(option: ProductOption, valueID: string): ProductOptionValue

    /** Returns a collection of product option values for the specified product option. */
    getOptionValues(option: ProductOption): Collection

    /** Returns the effective price of the specified option value. */
    getPrice(optionValue: ProductOptionValue): Money

    /**
     * Returns the selected value for the specified product option. If no value was explicitly selected,
     * the default option value is returned.
     */
    getSelectedOptionValue(option: ProductOption): ProductOptionValue

    /** Returns true if the specified option value is currently selected. */
    isSelectedOptionValue(option: ProductOption, value: ProductOptionValue): boolean

    /** Updates the selection of the specified option based on the specified value. */
    setSelectedOptionValue(option: ProductOption, value: ProductOptionValue): void

    /**
     * Returns a URL that can be used to select one or more option values. Accepts an action and
     * variable option/value pairs (options can be ProductOption or option ID; values can be
     * ProductOptionValue or value ID). Invalid pairs are ignored.
     */
    url(action: string, ...varOptionAndValues: Object[]): URL

    /** Returns a string URL that selects a specific value of a specific option. */
    urlSelectOptionValue(action: string, option: ProductOption, value: ProductOptionValue): string
 }
 ```
