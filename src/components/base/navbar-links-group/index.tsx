import { useState } from 'react';
import { Box, Collapse, Group, Indicator, Text, UnstyledButton } from '@mantine/core';
import classes from './index.module.css';
import { useNavigate } from 'react-router-dom';
import { useUpdateVersion, useVersion } from '@/store/about/hooks';


interface LinksGroupProps {
    icon: React.FC<any>;
    label: string;
    href?: string;
    active: string;
    initiallyOpened?: boolean;
    links?: { label: string; link: string, icon?: React.FC<any> }[];
    changeActive: (active: string) => void;
}

export function LinksGroup({ icon: Icon, label, href, initiallyOpened, links, active, changeActive: changeActive }: LinksGroupProps) {
    const hasLinks = Array.isArray(links) && links.length > 0;
    const [opened, setOpened] = useState(initiallyOpened || false);
    const navigate = useNavigate();
    const updateVersion = useUpdateVersion()
    const version = useVersion()
    const items = (hasLinks ? links : []).map((link) => (
        <Text<'a'>
            component="a"
            className={classes.link}
            data-active={link.link === active || undefined}
            href={link.link}
            key={link.label}
            onClick={(event) => event.preventDefault()}
        >
            <Box style={{ display: 'flex', alignItems: 'center' }} onClick={() => {
                changeActive(link.link)
                navigate(link.link)
            }}>
                {link.icon && <link.icon size={18} />}
                <Box ml="md">
                    <Text fz={"md"} fw={500}>
                        {link.label}
                    </Text>
                </Box>
            </Box>

        </Text>
    ));

    function onClickLink() {
        if (hasLinks) {
            setOpened((o) => !o)
        } else if (href) {
            changeActive(href)
            navigate(href)
        }
    }
    function checkckDisableIndicator() {
        if (label != "About") {
            return true
        } else if (updateVersion && version && version != updateVersion.version) {
            return false
        }
        return true
    }
    return (
        <>
            <UnstyledButton className={classes.control} onClick={onClickLink} data-active={href === active || undefined}>
                <Group justify="space-between" gap={0}>
                    <Box style={{ display: 'flex', alignItems: 'center' }}>
                        <Icon size={18} />
                        <Indicator inline color="red" offset={-1} size={6} processing disabled={checkckDisableIndicator()}>
                            <Box ml="md">
                                <Text fw={500} style={{ fontSize: 14 }}>
                                    {label}
                                </Text>
                            </Box>
                        </Indicator>

                    </Box>
                </Group>
            </UnstyledButton>
            {hasLinks ? <Collapse in={opened}>{items}</Collapse> : null}
        </>
    );
} 