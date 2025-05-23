import "./index.css";
import "@radix-ui/themes/styles.css";

import {
  Flex,
  Section,
  Theme,
  Container,
  Heading,
  Text,
} from "@radix-ui/themes";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ErrorBoundary } from "react-error-boundary";

import { Nav } from "./components/Nav";
import { ChainContextProvider } from "./context/ChainContextProvider";
import { RpcContextProvider } from "./context/RpcContextProvider";
import { SelectedWalletAccountContextProvider } from "./context/SelectedWalletAccountContextProvider";
import Root from "./routes/root";

function ErrorFallback({
  error,
  resetErrorBoundary,
}: {
  error: Error;
  resetErrorBoundary: () => void;
}) {
  console.error("Error caught by boundary:", error);
  return (
    <Container p="4">
      <Heading color="red">Something went wrong:</Heading>
      <Text as="pre" size="2" style={{ whiteSpace: "pre-wrap" }}>
        {error.message}
        {"\n\n"}
        {error.stack}
      </Text>
      <button onClick={resetErrorBoundary}>Try again</button>
    </Container>
  );
}

const rootNode = document.getElementById("root");
if (rootNode) {
  const root = createRoot(rootNode);
  root.render(
    <StrictMode>
      <Theme>
        <ErrorBoundary FallbackComponent={ErrorFallback}>
          <ChainContextProvider>
            <SelectedWalletAccountContextProvider>
              <RpcContextProvider>
                <Flex direction="column" minHeight="100vh">
                  <Nav />
                  <Section flexGrow="1">
                    <Root />
                  </Section>
                </Flex>
              </RpcContextProvider>
            </SelectedWalletAccountContextProvider>
          </ChainContextProvider>
        </ErrorBoundary>
      </Theme>
    </StrictMode>,
  );
} else {
  document.body.innerHTML =
    "<div style='color:red;padding:2rem'>No #root element found in index.html</div>";
}
