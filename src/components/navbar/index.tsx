
import { Box, Group, Space, Image, Flex } from '@mantine/core';
import { useEffect, useState } from 'react';
import classes from './navbar.module.css';
import { linkdata } from '../../routers';
import { LinksGroup } from '../base/navbar-links-group';
import { useLocation, useNavigate } from 'react-router-dom';
import SyncBlockCard from '../card/sync-block-card';
import RustySessionStore from '@/commands/store'; 
function Navbar() {
    const [active, setActive] = useState('');
    const location = useLocation();
    const navigate = useNavigate()
    useEffect(() => {
        if (location && location.pathname) {
            setActive(location.pathname)
            RustySessionStore.set("currentPage", location.pathname);
        }
    }, [location])
    useEffect(() => {
        navigateChange()
    }, [])

    async function navigateChange() {
        let currentPage = await RustySessionStore.get("currentPage");
        if (currentPage && currentPage !== location.pathname) {
            navigate(currentPage)
        } else if (!currentPage) {
            navigate("/wallet")
        }
    }

    const links = linkdata.map((item) => <LinksGroup active={active} changeActive={function (active: string): void {
        setActive(active)
    }} {...item} key={item.label} />);

    return (<Box>
        <Group visibleFrom='sm'>
            <nav data-tauri-drag-region className={classes.navbar}>
                <Space data-tauri-drag-region h={54} />
                <Flex justify={"center"} align={"center"}>
                    <Image
                        data-tauri-drag-region
                        style={{ cursor: "pointer" }}
                        src={"/icon-wallet.png"}
                        w={"100%"}
                        h={29}
                    />
                </Flex>
                <Space data-tauri-drag-region h={16} />
                <div
                    data-tauri-drag-region
                    className={classes.navbarMain}>
                    {links}
                </div>
                <div className={classes.footer}>
                    <SyncBlockCard />
                </div>
            </nav>
        </Group>
    </Box>)
}

export default Navbar;