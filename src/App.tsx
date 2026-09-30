import {
  Section,
  Container,
  Heading,
  Text,
  Flex,
  Button,
} from "@radix-ui/themes";
import { SymbolIcon } from "@radix-ui/react-icons";
import { useLoginState } from "./hooks";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

function WorkInProgress() {
  return (
    <Flex align="center" gap="2">
      <SymbolIcon />
      <Text>Work in progress</Text>
    </Flex>
  );
}

function Logged() {
  const { state, logout } = useLoginState();
  // Runs without problems but tauri-specta is needed to bind rust types to ts types.
  const [siteInfo, setSiteInfo] = useState<SiteInfo | null>(null);

  useEffect(() => {
    async function fetchSiteInfo() {
      try {
        const info = await invoke<SiteInfo>("get_site_info");
        setSiteInfo(info);
        console.log(info);
      } catch (error) {
        console.error("Failed to fetch site info:", error);
      }
    }

    fetchSiteInfo();
  }, []);

  return (
    <Flex direction="column">
      <Heading>
        Welcome {siteInfo?.firstname} {siteInfo?.lastname}
      </Heading>
      <WorkInProgress />
      <Flex align="center" gap="2" mt="4">
        <Button onClick={logout}>Logout</Button>
      </Flex>
    </Flex>
  );
}

function NotLogged() {
  const { login } = useLoginState();
  return (
    <Flex direction="column" align="start">
      <Heading>Welcome to Webeep-RS</Heading>
      <WorkInProgress />
      <Button mt="4" onClick={login}>
        Login
      </Button>
    </Flex>
  );
}

function Logging() {
  return (
    <Flex direction="column">
      <Heading>Logging in ...</Heading>
      <WorkInProgress />
    </Flex>
  );
}

function Error() {
  const { state } = useLoginState();
  return (
    <Flex direction="column">
      <Heading>An error occurred</Heading>
      <WorkInProgress />
      <Text mt="4">{state.error}</Text>
    </Flex>
  );
}

function App() {
  const { state } = useLoginState();

  return (
    <Section p="4">
      <Container>
        {state.state == "Logged" && <Logged></Logged>}
        {state.state == "Logging" && <Logging></Logging>}
        {state.state == "Error" && <Error></Error>}
        {state.state == "NotLogged" && <NotLogged></NotLogged>}
      </Container>
    </Section>
  );
}

export default App;
