import { Tab } from "@headlessui/react";
import init, { js_generate } from "agar-man";
import { Fragment, useEffect, useMemo, useState } from "react";
import { GrReturn } from "react-icons/gr";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List } from "react-window";
import { Poem } from "../components/Poem";

interface Rendered {
  seed: string;
  sentence: string;
}

const Results: React.VFC<{
  results: string[];
  setRendered: React.Dispatch<React.SetStateAction<Rendered>>;
  renderedSeed: string;
  renderedSentence: string;
}> = ({ results, setRendered, renderedSeed, renderedSentence }) => {
  return (
    <div className="h-full overflow-y-scroll">
      <AutoSizer>
        {({ height, width }) => (
          <List
            className="List"
            height={height}
            itemCount={results.length}
            itemSize={22}
            width={width}
            itemData={results}
          >
            {({ data, index, style }) => {
              const result = data[index];
              const split_result = result.split(" ");

              const isSelected = renderedSentence === result;

              return (
                <button
                  key={result}
                  className={`flex w-full px-2 text-sm text-neutral-500 hover:text-black hover:bg-neutral-200 ${
                    isSelected && "bg-neutral-200 text-black"
                  }`}
                  onClick={() => {
                    setRendered({ seed: renderedSeed, sentence: result });
                  }}
                  style={style}
                >
                  {split_result.map((word, index) => (
                    <div key={index} className="mr-1">
                      {word}
                    </div>
                  ))}
                </button>
              );
            }}
          </List>
        )}
      </AutoSizer>
    </div>
  );
};

const Input: React.VFC<{
  loading: boolean;
  generate: ({
    seed,
    minLength,
    maxWords,
    excludes,
    includes,
    topN,
  }: {
    seed: string;
    minLength: number;
    maxWords: number;
    excludes: string;
    includes: string;
    topN?: number;
  }) => void;
  renderedSeed: string;
}> = ({ loading, generate, renderedSeed }) => {
  const [seed, setSeed] = useState("anagram");

  const [minLength, setMinLength] = useState<number>(5);
  const [maxWords, setMaxWords] = useState<number>(5);
  const [excludes, setExcludes] = useState<string>("");
  const [includes, setIncludes] = useState<string>("");
  const [topN, setTopN] = useState<number>();

  const [lengthOptions, setLengthOptions] = useState<number[]>([]);
  const [maxWordsOptions, setMaxWordsOptions] = useState<number[]>([
    1, 2, 3, 4, 5, 6, 7, 8,
  ]);

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        generate({ seed, minLength, maxWords, excludes, includes, topN });
      }}
      className="flex flex-col"
    >
      <>
        <div className="flex w-full">
          <input
            spellCheck="false"
            autoCapitalize="none"
            autoCorrect="off"
            autoComplete="off"
            className="w-full px-2 py-1 border-b border-black outline-none"
            placeholder="Phrase to search..."
            type="text"
            value={seed}
            disabled={loading}
            onChange={(e) => {
              const newLengthOptions = [];

              for (
                let i = 2;
                i < Math.ceil(e.target.value.length / 2) + 1;
                i++
              ) {
                newLengthOptions.push(Math.floor(i));
              }

              setLengthOptions(newLengthOptions);

              if (minLength === null || minLength === undefined) {
                setMinLength(newLengthOptions[0]);
              } else {
                if (minLength < newLengthOptions[0]) {
                  setMinLength(newLengthOptions[0]);
                }
                if (minLength > newLengthOptions[newLengthOptions.length - 1]) {
                  setMinLength(newLengthOptions[newLengthOptions.length - 1]);
                }
              }

              let cleanedSeed = e.target.value
                .replace(/[^a-zA-Z]\s/g, "")
                .toLowerCase();

              setSeed(cleanedSeed);
            }}
          />
          <div className="flex items-center border-b border-black">
            <input
              className="h-full px-2 border-black border-l w-[11rem] outline-none"
              value={topN}
              onChange={(e) => {
                let cleaned = e.target.value.replace(/[^0-9]/g, "");
                setTopN(parseInt(cleaned) || undefined);
              }}
              placeholder="Most common words"
            />
            <div className="px-2 border-black">
              <span className="text-neutral-400">/</span>
              <span className="">178,691</span>
            </div>
          </div>
        </div>
        <div className="border-b border-black flex items-center">
          <div className="px-2 border-r border-neutral-300 h-full flex items-center w-24">
            Include
          </div>
          <input
            spellCheck="false"
            autoCapitalize="none"
            autoCorrect="off"
            autoComplete="off"
            className="w-full px-2 py-1 outline-none"
            placeholder="Must contain (comma separated)..."
            type="text"
            value={includes}
            disabled={loading}
            onChange={(e) => {
              let cleaned = e.target.value
                .replace(/[^a-zA-Z],\s/g, "")
                .toLowerCase();

              setIncludes(cleaned);
            }}
          />
        </div>
        <div className="border-b border-black flex items-center">
          <div className="px-2 border-r border-neutral-300 h-full flex items-center w-24">
            Exclude
          </div>
          <input
            spellCheck="false"
            autoCapitalize="none"
            autoCorrect="off"
            autoComplete="off"
            className="w-full px-2 py-1 outline-none"
            placeholder="Words to exclude (comma separated)..."
            type="text"
            value={excludes}
            disabled={loading}
            onChange={(e) => {
              let cleaned = e.target.value
                .replace(/[^a-zA-Z],\s/g, "")
                .toLowerCase();

              setExcludes(cleaned);
            }}
          />
        </div>
      </>
      {lengthOptions.length > 0 && (
        <div className="flex flex-wrap w-full text-sm border-b border-black select-none">
          <div className="px-2 py-1 border-r border-neutral-300">
            Minimum word length
          </div>
          {lengthOptions.map((length, index) => {
            return (
              <button
                key={length}
                onClick={() => {
                  setMinLength(length);
                  // generate({ seed, minLength: length, maxWords });
                }}
                className={`px-2 py-1 text-sm border-r ${
                  length === minLength && "bg-black text-white"
                }`}
              >
                {length} {index === lengthOptions.length - 1 && "+"}
              </button>
            );
          })}
        </div>
      )}
      {maxWordsOptions.length > 0 && (
        <div className="flex flex-wrap w-full text-sm select-none">
          <div className="px-2 py-1 border-r  border-neutral-300">
            Max words
          </div>
          {maxWordsOptions.map((max, index) => {
            return (
              <button
                key={max}
                onClick={() => {
                  setMaxWords(max);
                  // generate({ seed, minLength, maxWords: max });
                }}
                className={`px-2 py-1 text-sm border-r border-neutral-300 ${
                  maxWords === max && "bg-black text-white"
                }`}
              >
                {max}
              </button>
            );
          })}
        </div>
      )}
      <button
        type="submit"
        className="px-2 flex justify-center items-center border-t py-1 border-black hover:bg-neutral-100 select-none"
        disabled={loading}
        onClick={() => {
          generate({ seed, minLength, maxWords, excludes, includes, topN });
        }}
      >
        Generate
        <span className="px-1 py-0.5 ml-2 border-neutral-300 bg-neutral-200s border">
          <GrReturn />
        </span>
      </button>
    </form>
  );
};

export const Search = () => {
  const [renderedSeed, setRenderedSeed] = useState("anagram");

  const [loading, setLoading] = useState(false);

  const [activeTab, setActiveTab] = useState(0);

  const [results, setResults] = useState<string[]>([]);
  const [partials, setPartials] = useState<string[]>([]);

  const [rendered, setRendered] = useState<Rendered>({
    seed: "anagram",
    sentence: "agar man",
  });

  const [executionTime, setExecutionTime] = useState(0);

  useEffect(() => {
    generate({
      seed: "anagram",
      minLength: 3,
      maxWords: 5,
      excludes: "",
      includes: "",
      topN: 200_000,
    });
  }, []);

  const generate = ({
    seed,
    minLength,
    maxWords,
    includes,
    excludes,
    topN,
  }: {
    seed: string;
    minLength: number;
    maxWords: number;
    includes: string;
    excludes: string;
    topN?: number;
  }) => {
    setLoading(true);
    init().then(() => {
      const start = window.performance.now();
      const { anagrams, partials } = js_generate(
        seed.toLowerCase(),
        minLength,
        maxWords,
        excludes,
        includes,
        topN || 200_000
      );
      setResults([...anagrams]);
      setPartials(
        [...partials].sort((a, b) => {
          return b.length - a.length;
        })
      );
      setRenderedSeed(seed);
      const end = window.performance.now();
      setExecutionTime(Math.floor(end - start));
      setLoading(false);
    });
  };

  return (
    <div className="sm:p-2 md:p-4 p-0 h-screen flex w-full">
      <div className="flex flex-col w-full max-w-screen-lg mx-auto">
        <div className="flex flex-col border-black border">
          <Input
            loading={loading}
            generate={generate}
            renderedSeed={renderedSeed}
          />
        </div>
        <div className="flex w-full grow border mt-4 border-black">
          <div className="flex flex-col w-1/2 border-r border-black xs:w-1/2 sm:w-1/3 shrink-0">
            <div className="px-2 pt-1 pb-1 text-sm flex flex-col text-neutral-400 border-b border-black select-none">
              {loading ? (
                <>Loading...</>
              ) : (
                <>
                  {results.length.toLocaleString("en-US")} results in{" "}
                  {(executionTime / 1000).toLocaleString("en-US")}s for "
                  {renderedSeed}"
                </>
              )}
              <button
                className="border-black max-w-max border px-2 mt-1 text-black hover:bg-neutral-100 select-none"
                onClick={() => {
                  const text = results.join("\n");
                  const file = new Blob([text], {
                    type: "text/plain",
                  });
                  const element = document.createElement("a");
                  element.href = URL.createObjectURL(file);
                  element.download = `${renderedSeed}.txt`;
                  document.body.appendChild(element); // Required for this to work in FireFox
                  element.click();
                }}
              >
                Download as .txt
              </button>
            </div>

            {!loading && (
              <Results
                results={results}
                setRendered={setRendered}
                renderedSeed={renderedSeed}
                renderedSentence={rendered.sentence}
              />
            )}
          </div>
          <div className="flex flex-col items-center justify-center mx-auto">
            {rendered !== null && (
              <>
                <Poem
                  key={rendered.sentence}
                  seed={rendered.seed}
                  sentence={rendered.sentence}
                  showPerms
                />
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
