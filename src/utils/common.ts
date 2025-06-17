import BigNumber from 'bignumber.js';
// +
export function bigNumberPlus(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.plus(b).toNumber();  
}
// +
export function bigNumberPlusToString(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.plus(b).toFixed().toString();   
}
// -
export function bigNumberMinus(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.minus(b).toNumber();
}
// *
export function bigNumberTimes(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.times(b).toNumber();
}
// *
export function bigNumberTimesToString(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.times(b).toFixed().toString();
}
// /
export function bigNumberDiv(a: any, b: any) {
    a = BigNumber(a);
    b = BigNumber(b);
    return a.div(b).toNumber();
}

function isPositiveNumber(numberStr: string) {
    const num = parseFloat(numberStr);
    if (isNaN(num)) {
        return false;
    }
    return num < 0 ? false : true;
} 
