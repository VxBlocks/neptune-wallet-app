import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { ContactState } from "../types";
import { Contact } from "@/database/types/contact";
import { getWallets } from "@/commands/wallet";
import { getContactList } from "@/utils/storage";
const initialState: ContactState = {
    loadingContacts: false,
    contacts: []
}
const contactSlice = createSlice({
    name: "contact",
    initialState,
    reducers: {

    },
    extraReducers: (builder) => {
        builder.addCase(queryAllContacts.pending, (state) => {
            state.loadingContacts = true;
        });
        builder.addCase(queryAllContacts.fulfilled, (state, action) => {
            state.loadingContacts = false;
            state.contacts = action.payload.data;
        });
        builder.addCase(queryAllContacts.rejected, (state) => {
            state.loadingContacts = false;
        });
    }
})


export const queryAllContacts = createAsyncThunk<
    { data: Contact[] }
>(
    '/api/contact/queryAllContacts',
    async () => {
        const contactList = await getContactList();
        let newContactList = await merageAddress(contactList);
        return {
            data: newContactList
        }
    }
)

async function merageAddress(contactList: Contact[]) {
    let newContactList = contactList ?? [] as Contact[];
    try {
        const res = await getWallets();
        if (res && res.length > 0) {
            for (let i = 0; i < res.length; i++) {
                let ownerWallet = res[i];
                newContactList.push({
                    aliasName: ownerWallet.name,
                    address: ownerWallet.address,
                    type: "owner",
                    remark: "",
                    createdTime: 0,
                })
            }
        }
    } catch (error) {

    }

    return newContactList
}


export const {
} = contactSlice.actions;

export default contactSlice.reducer;