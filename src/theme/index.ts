import { createTheme, MantineColorsTuple, rem } from "@mantine/core";
const myColor: MantineColorsTuple = [
    "#edefff",
    "#d9dbfa",
    "#b1b4ec",
    "#868bde",
    "#6268d3",
    "#535acf",
    "#3e46ca",
    "#3038b4",
    "#2832a2",
    "#1d2a90"
];
const theme = createTheme({
    primaryColor: 'myColor',
    colors: {
        myColor,
        // or replace default theme color
        blue: [
            '#eef3ff',
            '#dee2f2',
            '#bdc2de',
            '#98a0ca',
            '#7a84ba',
            '#6672b0',
            '#5c68ac',
            '#4c5897',
            '#424e88',
            '#364379',
        ],

    },
    headings: {
        sizes: {
            h1: { fontSize: rem(36) },
        },
    },
    components: {
        Modal: {
            styles: {
                content: {
                    borderRadius: 20,
                },
                body: {
                    padding: "20px 20px 30px 20px",
                },
                title: {
                    fontWeight: 700,
                    fontSize: 18,
                },
            }
        }, Button: {
            styles: (theme: any) => ({
                root: {
                    backgroundColor: `${theme.colors.blue[6]} !important`, 
                },
            }),
        },
        Text: {
            styles: () => ({
                root: {
                    wordWrap: 'break-word', 
                    fontWeight: 500,
                    fontSize: '14px',
                },
            })
        }
    }
});


export default theme;