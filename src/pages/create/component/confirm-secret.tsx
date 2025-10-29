import { set_password } from "@/commands/password";
import { addWallet } from "@/commands/wallet";
import { useAppDispatch } from "@/store/hooks";
import { useMnemonic, useOneTimePassword, useOneTimeWalletName } from "@/store/wallet/hooks";
import { setMnemonic, setOneTimePassword } from "@/store/wallet/wallet-slice";
import { Box, Button, Flex, Grid, Text } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { IconX } from "@tabler/icons-react";
import { useEffect, useState } from "react";
interface Props {
    nextStep: () => void;
}
export default function ConfirmSecret(props: Props) {
    const { nextStep } = props
    const walletName = useOneTimeWalletName()
    const oneTimePassword = useOneTimePassword()
    const mnemonic = useMnemonic()
    const [numbers, setNumbers] = useState([] as number[]);
    const [loading, setLoading] = useState(false) 
    const dispatch = useAppDispatch()
    useEffect(() => {
        generateRandomNumbers()
    }, [mnemonic])

    const generateRandomNumbers = () => {
        // Generate an array of numbers from 0 to 17
        const arr = Array.from({ length: 18 }, (_, i) => i);

        // Fisher-Yates shuffle algorithm
        for (let i = arr.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [arr[i], arr[j]] = [arr[j], arr[i]];
        }
        // Take the first 5 unique numbers
        setNumbers(arr.slice(0, 5));
    };
    const [inputWords, setInputWords] = useState([] as string[])

    const [verifyWords, setVverifyWords] = useState([] as string[])

    useEffect(() => {
        if (numbers && numbers.length === 5 && mnemonic) {
            generateRandomWords()
        }
    }, [numbers, mnemonic])

    function generateRandomWords() {
        let words = [] as string[]
        let mnemonicList = mnemonic.split(" ")
        let newMnemonicList = [] as string[] 
        numbers.forEach((item) => {
            let findWord = mnemonicList.find((item2, index2) => index2 == item)
            if (findWord) {
                words.push(findWord)
            }
        })
        // Fisher-Yates shuffle algorithm
        for (let i = words.length - 1; i > 0; i--) {
            const j = Math.floor(Math.random() * (i + 1));
            [words[i], words[j]] = [words[j], words[i]];
        } 
        mnemonicList.forEach((item, index) => {
            const exists = numbers.includes(index); // true
            let word = item
            if (exists) {
                word = ""
            }
            newMnemonicList.push(word)
        })   
        setVverifyWords(newMnemonicList)
        setInputWords(words);
    }

    function selecteWord(index: number, word: string) {
        let newInputWords = [] as string[]
        let newVerifyWords = [] as string[] 
        inputWords.forEach((item) => {
            if (item != word) {
                newInputWords.push(item)
            }
        })
        let addOne = false 
        verifyWords.forEach((item) => {
            if (!addOne) {
                if (item === "") {
                    addOne = true
                    item = word
                }
            }
            newVerifyWords.push(item)
        })
        setVverifyWords(newVerifyWords)
        setInputWords(newInputWords)
    }

    function removeWord(index: number, word: string) {
        let newInputWords = [] as string[]
        let newVerifyWords = [] as string[] 
        verifyWords.forEach((item, itemIndex) => {
            if (index == itemIndex) {
                item = ""
            }
            newVerifyWords.push(item)
        }) 
        newInputWords = [...inputWords, word]
        setVverifyWords(newVerifyWords)
        setInputWords(newInputWords)
    }


    async function checkSecret() { 
        if (verifyWords.join(" ") != mnemonic) {
            notifications.show({
                position: "top-right",
                message: "The recovery phrase is incorrect, please check again.",
                color: "red",
                title: "Error",
            })
            return;
        }
        setLoading(true)
        try {
            await set_password("", oneTimePassword)
            await addWallet(walletName, mnemonic, 25, 0, true)  
          dispatch(setMnemonic("")) 
          dispatch(setOneTimePassword(""))
            nextStep()
        } catch (error: any) { 
            notifications.show({
                position: "top-right",
                message: error || "Failed to add wallet, please try again later.",
                color: "red",
                title: "Error",
            })
        }
        setLoading(false)
    }

    return (<Flex direction="column" justify={"center"} align="center" gap={8} w={"100%"}>
        <Box style={{ width: "100%", border: "1px solid #000000", borderRadius: "5px", padding: "16px", caretColor: "transparent", }}>
            <Grid>
                {verifyWords && verifyWords.length > 0 && verifyWords.map((word, index) => {
                    return (<Grid.Col span={4} key={index}>
                        <Flex direction={"row"} justify={"center"} align={"center"} gap={8}>
                            <Text style={{ minWidth: "18px", textAlign: "center" }} fw={"bold"}>{`${index + 1}.`}</Text>
                            <Flex direction={"row"} style={{
                                border: "1px solid #000000",
                                borderRadius: "5px",
                                padding: "4px",
                                minWidth: "120px",
                                minHeight: "32px",
                                caretColor: "transparent",
                            }} justify={"center"} align={"center"}>
                                <Text style={{ fontWeight: "bold" }}>
                                    {word}
                                </Text>
                                {
                                    word && numbers.includes(index) && <IconX size={14} style={{ cursor: "pointer" }} onClick={() => removeWord(index, word)} />
                                }
                            </Flex>
                        </Flex>
                    </Grid.Col>)
                })}
            </Grid>
        </Box>
        <Box style={{ width: "100%", padding: "16px", caretColor: "transparent", minHeight: "120px" }}>
            <Grid>
                {
                    inputWords && inputWords.map((word, index) => {
                        return <Grid.Col span={4} key={index} style={{ cursor: "pointer" }} onClick={() => selecteWord(index, word)}>
                            <Flex direction={"row"} justify={"center"} align={"center"} gap={8}>
                                <Text fw={"bold"} style={{ color: "transparent" }}>{`${index + 1}.`}</Text>
                                <Flex style={{
                                    border: "2px solid #000000",
                                    borderRadius: "5px",
                                    padding: "4px",
                                    minWidth: "120px",
                                    caretColor: "transparent",
                                }} justify={"center"}>
                                    <Text style={{ fontWeight: "bold" }}>
                                        {word}
                                    </Text>
                                </Flex>
                            </Flex>
                        </Grid.Col>
                    })
                }
            </Grid>
        </Box>
        <Flex justify={"center"} align={"center"} w={"100%"}>
            <Button variant="light" fullWidth disabled={inputWords && inputWords.length != 0} loading={loading} onClick={checkSecret}>
                Confirm Secret Recovery Phrase
            </Button>
        </Flex>
    </Flex>)
}
