import { bigNumberDiv, bigNumberTimesToString } from "./common";

export function amount_to_fixed(amount: string) {
    if (!amount) return "0"
    let len = amount.length;
    return amount.substring(0, len - 30)
}

export function amount_to_positive_fixed(amount: string) {
    if (!amount) return "0"
    let new_amount = amount.replace("-", "")
    // 1. Math truncate: scale up -> truncate -> scale down
    let aa = bigNumberTimesToString(new_amount, 10000)
    let truncatedValue = bigNumberDiv(Math.trunc(aa), 10000)

    // 2. Convert to string with exactly 4 decimal places (zero-padded)
    return truncatedValue.toLocaleString('fullwide', {
        useGrouping: false,       // Disable thousand separators
        minimumFractionDigits: 4, // Minimum number of decimal digits
        maximumFractionDigits: 4  // Maximum number of decimal digits
    });
}
