import { useEffect, useState } from "react";
import CopyToClipboard from "react-copy-to-clipboard";
import { FaCheck } from "react-icons/fa";
import { FiLink } from "react-icons/fi";

export const Poem: React.VFC<{ seed: string; sentence: string[] }> = ({
  seed,
  sentence,
}) => {
  const charPositions: { [key: string]: number[] } = {};

  seed.split("").forEach((letter, index) => {
    if (letter in charPositions) {
      charPositions[letter].push(index);
    } else {
      charPositions[letter] = [index];
    }
  });

  const [copied, setCopied] = useState(false);

  useEffect(() => {
    setCopied(false);
  }, [seed, sentence]);

  return (
    <div className="flex flex-col items-center justify-center">
      <div className="flex">
        {sentence.map((word, index) => (
          <div className={`flex ${index > 0 && "ml-4"}`} key={index}>
            {word.split("").map((letter, index) => {
              const letterIndex = charPositions[letter].shift()!;
              const offset = seed.length - letterIndex;
              const filler = new Array(offset).fill("x");
              return (
                <div className="flex flex-col" key={index}>
                  {filler.map((x, index) => {
                    return (
                      <div key={index} className="leading-5">
                        &nbsp;
                      </div>
                    );
                  })}
                  {seed.split("").map((letter, index) => {
                    return (
                      <div
                        key={index}
                        className={`w-6 leading-5 text-center ${
                          index !== letterIndex && "text-gray-300"
                        }`}
                      >
                        {letter === " " ? <span>&nbsp;</span> : letter}
                      </div>
                    );
                  })}
                </div>
              );
            })}
          </div>
        ))}
      </div>

      <CopyToClipboard
        text={`${window.location.origin}/share/${seed}/${sentence.join(",")}`}
        onCopy={() => setCopied(true)}
      >
        <button className="p-2 mt-6 hover:bg-gray-100">
          {copied ? <FaCheck className="text-green-500" /> : <FiLink />}
        </button>
      </CopyToClipboard>
    </div>
  );
};
