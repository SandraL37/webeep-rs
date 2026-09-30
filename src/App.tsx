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
import { commands, LoginError, SiteInfo } from "./bindings";

function WorkInProgress() {
  return (
    <Flex align="center" gap="2">
      <SymbolIcon />
      <Text>Work in progress</Text>
    </Flex>
  );
}

function Logged({ token }: { token: string }) {
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
        <Button onClick={commands.logout}>Logout</Button>
      </Flex>
    </Flex>
  );
}

function NotLogged() {
  return (
    <Flex direction="column" align="start">
      <Heading>Welcome to Webeep-RS</Heading>
      <WorkInProgress />
      <Button mt="4" onClick={commands.login}>
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

function Error({ error }: { error: LoginError }) {
  return (
    <Flex direction="column">
      <Heading>An error occurred</Heading>
      <WorkInProgress />
      <Text mt="4">{error.toString()}</Text>
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
