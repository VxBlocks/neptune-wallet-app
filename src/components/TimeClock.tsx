import { CSSProperties, useState, useEffect } from "react"

/**
 * Timer component. Pass in a timestamp, currently only accepts second-level timestamps. Millisecond support can be added later if needed.
 */
interface clockProps {
    timeStamp: number
    style?: CSSProperties | undefined
}

export const TimeClock = (props: clockProps) => {
    const { timeStamp, style } = props
    const [ts, setTS] = useState(0)
    const [value, setValue] = useState("")

    // Timer function
    function updateTime() {
        setTS((prev) => prev + 1)
    }

    useEffect(() => {
        const currentTS = Math.floor(Date.now() / 1000)
        const ts = currentTS - timeStamp
        setTS(ts)
    }, [timeStamp])

    useEffect(() => {
        // Execute the update timer function every second
        let timer = setInterval(updateTime, 1000)
        // Triggered when the component is destroyed, clean up unused timers and release system resources
        return () => {
            clearInterval(timer)
        }
    }, [])

    useEffect(() => {
        let minute = Math.trunc(ts / 60)
        let second = ts % 60
        if (minute <= 0) {
            setValue(`${getSec(second)}`)
        } else {
            setValue(`${getMin(minute)} ${getSec(second)}`)
        }
    }, [ts])

    function getSec(second: number) {
        return `${second} s ago`
    }

    function getMin(minute: number) {
        return `${minute} m`
    }

    return (
        <span style={{ ...style }}>
            {value}
        </span>
    )
}