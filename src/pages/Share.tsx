import {useParams} from 'react-router-dom';
import { Poem } from '../components/Poem';
import { CopyToClipboard } from "react-copy-to-clipboard";
import { useState } from 'react';
import {FiLink} from 'react-icons/fi';
import { FaCheck } from "react-icons/fa";

export const Share = () => {
    const {seed, components} = useParams();

    return (
      <div className="flex items-center justify-center w-screen h-screen">
        <div className="flex flex-col items-center justify-center p-12 border border-black">
          <div>
            {seed && components && (
              <Poem seed={seed} sentence={components?.split(",")} />
            )}
          </div>
        </div>
      </div>
    );
}