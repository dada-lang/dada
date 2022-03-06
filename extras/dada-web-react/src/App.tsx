import React from "react";
import Container from "react-bootstrap/Container";

import Footer from "./components/footer";
import Navbar from "./components/navbar";
import Ide from "./features/ide";

function App() {
  return (
    <>
      <header id="nav-header">
        <Navbar />
      </header>
      <main id="content" className="mx-md-5">
        <Container fluid>
          <Ide />
        </Container>
      </main>
      <footer className="footer mt-auto py-1 mx-2 mx-md-5">
        <Footer />
      </footer>
    </>
  );
}

export default App;
