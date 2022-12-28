using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace YUAUY_Wallpaper.Image
{
    public class ImagePicker
    {
        public ImagePicker()
        {
            ImageFolderList = new List<string>();
            Sort = ImagePickerSort.Name;
        }
        public List<string> ImageFolderList { get; private set; }
        public ImagePickerSort Sort { get; set; }
        public void Reset()
        {
            _index = -1;
        }

        private int _index = -1;
        private IEnumerable<string> _imagePaths;
        public Bitmap? Next()
        {
            _index++;
            if (_index > _imagePaths.Count()) _index = 0;

            if (_imagePaths.Count() != ImageFolderList.Sum(path => Directory.GetFiles(path).Length) || // Tmp Paths Count != Now Paths Count
              !File.Exists(_imagePaths.ElementAt(_index))) // or file not exists
            {
                _Refresh_imagePaths();
                if (_index > _imagePaths.Count()) _index = 0;
            }

            if (_imagePaths.Count() > 0)
                return new Bitmap(File.OpenRead(_imagePaths.ElementAt(_index)));
            else
                return null;
        }

        private int _randomSeed = (int)DateTime.Now.Ticks;
        private void _Refresh_imagePaths()
        {
            _CheckAllFolder();
            _imagePaths = ImageFolderList.SelectMany(folderPath => Directory.GetFiles(folderPath));
            switch (Sort)
            {
                case ImagePickerSort.Name:
                    _imagePaths = _imagePaths.OrderBy(path => path);
                    break;
                case ImagePickerSort.Name_Reverse:
                    _imagePaths = _imagePaths.OrderByDescending(path => path);
                    break;
                case ImagePickerSort.CreationTime:
                    _imagePaths = _imagePaths.OrderBy(path => File.GetCreationTime(path));
                    break;
                case ImagePickerSort.CreationTime_Reverse:
                    _imagePaths = _imagePaths.OrderByDescending(path => File.GetCreationTime(path));
                    break;
                case ImagePickerSort.LastWriteTime:
                    _imagePaths = _imagePaths.OrderBy(path => File.GetLastWriteTime(path));
                    break;
                case ImagePickerSort.LastWriteTime_Reverse:
                    _imagePaths = _imagePaths.OrderByDescending(path => File.GetLastWriteTime(path));
                    break;
                case ImagePickerSort.Random:
                    _randomSeed = (int)DateTime.Now.Ticks;
                    Random random = new Random(_randomSeed);
                    _imagePaths = _imagePaths.OrderBy(path => random.Next(2));
                    break;
                case ImagePickerSort.Random_NotRepeating:
                    Random random2 = new Random(_randomSeed);
                    _imagePaths = _imagePaths.OrderBy(path => random2.Next(2));
                    break;
            }
        }
        private void _CheckAllFolder()
        {
            ImageFolderList.RemoveAll(folderPath => !Directory.Exists(folderPath));
        }

    }
}
