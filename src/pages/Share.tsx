import {useParams} from 'react-router-dom';
import { Poem } from '../components/Poem';

export const Share = () => {
    const {seed, components} = useParams();
    return (
      <div className="flex items-center justify-center w-screen h-screen">
        <div className="p-8 border border-black">
          {seed && components && (
            <Poem seed={seed} sentence={components?.split(",")} />
          )}
        </div>
      </div>
    );
}