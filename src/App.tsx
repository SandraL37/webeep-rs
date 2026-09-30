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
import { useEffect, useState } from "react";
import { commands, SiteInfo } from "./bindings";

function WorkInProgress() {
  return (
    <Flex align="center" gap="2">
      <SymbolIcon />
      <Text>Work in progress</Text>
    </Flex>
  );
}

function Logged({ token }: { token: string }) {
  const { logout } = useLoginState();
  const [siteInfo, setSiteInfo] = useState<SiteInfo | null>(null);

  useEffect(() => {
    async function fetchSiteInfo() {
      const info = await commands.getSiteInfo();
      if (info.status == "ok") {
        setSiteInfo(info.data);
      } else {
        console.log(info);
      }
    }

    fetchSiteInfo();
  }, []);

  return (
    <Flex direction="column">
      <Heading>Welcome {siteInfo?.fullname}</Heading>
      <Text>token: {token}</Text>
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

function Error({ error }: { error: string }) {
  return (
    <Flex direction="column">
      <Heading>An error occurred</Heading>
      <WorkInProgress />
      <Text mt="4">{error}</Text>
    </Flex>
  );
}

function App() {
  const { state } = useLoginState();

  return (
    <Section p="4">
      <Container>
        {typeof state === "object" && state.Logged != null && (
          <Logged token={state.Logged.token}></Logged>
        )}
        {state == "NotLogged" && <NotLogged></NotLogged>}
        {state == "Logging" && <Logging></Logging>}
        {typeof state === "object" && state.Error != null && (
          <Error error={state.Error.error}></Error>
        )}
      </Container>
    </Section>
  );
}

export default App;
