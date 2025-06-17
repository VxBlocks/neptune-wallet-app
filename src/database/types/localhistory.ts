import { SendInputItem } from "@/utils/api/types.ts";

export interface ExecutionHistory {
    txid: string;
    timestamp: number;
    height: number;
    addressId: number;
    address: string;
    fee: string;
    priorityFee: string;
    status?: string;
    outputs: string[];
    batchOutput: SendInputItem[]
}

export interface ExecutionDbHistory {
    txid: string;
    timestamp: number;
    height: number;
    addressId: number;
    address: string;
    fee: string;
    priorityFee: string;
    status?: string; 
    batchOutput: string
}


