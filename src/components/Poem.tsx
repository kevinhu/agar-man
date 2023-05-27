import { useEffect, useMemo, useState } from "react";
import CopyToClipboard from "react-copy-to-clipboard";
import { FaCheck } from "react-icons/fa";
import { FiLink } from "react-icons/fi";

function getPermutations<T>(arr: T[]): T[][] {
  if (arr.length <= 1) return [arr];
  const perms = [];
  for (let i = 0; i < arr.length; i++) {
    const rest = arr.slice(0, i).concat(arr.slice(i + 1));
    for (const perm of getPermutations(rest)) {
      perms.push([arr[i], ...perm]);
    }
  }
  return perms;
}

export const Poem: React.VFC<{ seed: string; sentence: string }> = ({
  seed,
  sentence,
}) => {
  const charPositions: { [key: string]: number[] } = {};

  const [displayedSentence, setDisplayedSentence] = useState(sentence);

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

  const perms = useMemo(() => getPermutations(sentence.split(" ")), [sentence]);

  return (
    <div className="flex flex-col items-center justify-center">
      <div className="flex">
        {displayedSentence.split(" ").map((word, index) => (
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
                          index !== letterIndex && "text-neutral-300"
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
        text={`${window.location.origin}/share/${seed}/${displayedSentence.replace(" ",",")}`}
        onCopy={() => setCopied(true)}
      >
        <button className="p-2 mt-6 hover:bg-gray-100">
          {copied ? <FaCheck className="text-green-500" /> : <FiLink />}
        </button>
      </CopyToClipboard>
      <div className="flex items-center flex-col mt-8">
        <h1>Permutations</h1>
        <div className="flex flex-wrap text-neutral-400 gap-4 justify-center items-center p-4">
          {perms.map((perm, index) => {

            const selected = perm.join(" ") === displayedSentence

            return (
              <button
                key={index}
                className={`flex space-x-1 ${selected && 'text-black bg-neutral-200'} px-1 hover:text-black hover:bg-neutral-200`}
                onClick={() => {
                  setDisplayedSentence(perm.join(" "));
                }}
              >
                {perm.map((word, index) => {
                  return <div>{word}</div>;
                })}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
};
