import { persist_store_execute } from "@/commands/app";
import { Contact } from "@/database/types/contact";
import { ExecutionDbHistory, ExecutionHistory } from "@/database/types/localhistory";
import { notifications } from "@mantine/notifications";

export async function addContactAddress({ contact }: { contact: Contact }): Promise<boolean> {
    let success = false
    try {
        let sql = `INSERT INTO contacts (aliasName, address, type, remark, createdTime) VALUES ('${contact.aliasName}','${contact.address}','${contact.type}','${contact.remark}',${contact.createdTime})`
        await persist_store_execute(sql)
        success = true
    } catch (error) {
        throw (error)
    }
    return success
}

export async function deleteContactAddress({ address }: { address: string }): Promise<boolean> {
    let success = false
    try {
        let sql = `DELETE FROM contacts WHERE address = '${address}'`
        await persist_store_execute(sql)
        success = true
    } catch (error) {
        throw (error)
    }
    return success
}

export async function getContactList(): Promise<Contact[]> {
    let sql = `SELECT * FROM contacts`
    let contactList = [] as Contact[]
    try {
        let req = await persist_store_execute(sql)
        if (req && req.length > 0) {
            contactList = req as unknown as Contact[]
        }
    } catch (error) {
        throw (error)
    }
    return contactList
}


export async function getExecutionHistory({ addressId }: { addressId: number }): Promise<ExecutionHistory[]> {
    let sql = `SELECT * FROM execution_history 
WHERE addressId = '${addressId}'
ORDER BY timestamp DESC`
    let historys = [] as ExecutionHistory[]
    try {
        let req = await persist_store_execute(sql)
        let list = req as unknown as ExecutionDbHistory[]
        list.map((item) => {
            historys.push({ ...item, outputs: item.status && item.status != "" && item.status != "[]" && item.status != "undefined" ? JSON.parse(item.status) : [], batchOutput: item.batchOutput && item.batchOutput != "" && item.batchOutput != "[]" && item.batchOutput != "undefined" ? JSON.parse(item.batchOutput) : [] })
        })
    } catch (error) {
        throw (error)
    }
    return historys
}

export async function addExecutionHistory({ localHistory }: { localHistory: ExecutionHistory }) {
    let sql = `INSERT INTO execution_history (
    txid, 
    timestamp, 
    height, 
    addressId, 
    address, 
    fee, 
    priorityFee, 
    status,  
    batchOutput
) VALUES (
    '${localHistory.txid}', 
    ${localHistory.timestamp}, 
    ${localHistory.height}, 
    ${localHistory.addressId}, 
    '${localHistory.address}', 
    '${localHistory.fee}', 
    '${localHistory.priorityFee}',   
    '${localHistory.outputs.length > 0 ? JSON.stringify(localHistory.outputs) : ""}',  
    '${localHistory.batchOutput.length > 0 ? JSON.stringify(localHistory.batchOutput) : ""}'
);`
    let success = false
    try {
        await persist_store_execute(sql)
        success = true
    } catch (error: any) {
        console.log(error);
        notifications.show({
            position: "top-right",
            message: error,
            color: "red",
            title: "Error",
        })
    }
    return success
}

export async function deleteExecutionHistory({ txid }: { txid: string }): Promise<boolean> {
    let sql = `DELETE FROM execution_history WHERE txid = '${txid}'`
    let success = false;
    try {
        await persist_store_execute(sql)
        success = true
    } catch (error: any) {
        notifications.show({
            position: "top-right",
            message: error,
            color: "red",
            title: "Error",
        })
    }
    return success
}  