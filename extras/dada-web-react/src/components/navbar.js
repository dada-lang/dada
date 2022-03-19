import Container from "react-bootstrap/Container";
import Navbar from "react-bootstrap/Navbar";

function DadaNavbar() {
  return (
    <Navbar expand="md">
      <Container fluid>
        <div className="flex-grow-0 flex-shrink-1 mb-1">
          <Navbar.Toggle aria-controls="basic-navbar-nav" />
        </div>
        <Navbar.Collapse id="basic-navbar-nav">
          <div className="flex-grow-0 flex-shrink-1 d-none d-md-inline">
            <Navbar.Brand>
              <img
                src="/logo512.png"
                width="30"
                height="30"
                className="d-inline-block align-top"
                alt="Dada logo"
              />
              Dada
            </Navbar.Brand>
          </div>
        </Navbar.Collapse>
      </Container>
    </Navbar>
  );
}

export default DadaNavbar;
