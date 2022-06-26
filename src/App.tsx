import React from "react";
import { BrowserRouter, Route, Routes } from "react-router-dom";

import { Search } from "./pages/Search";
import { Share } from "./pages/Share";

const App = () => {
  return (
    <React.StrictMode>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Search />} />
          <Route path="/share/:seed/:components" element={<Share />} />
        </Routes>
      </BrowserRouter>
    </React.StrictMode>
  );
};

export default App;
