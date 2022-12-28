using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Drawing.Printing;
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
        public ImagePickerSort Sort
        {
            get
            {
                return _sort;
            }
            set
            {
                if (_sort == value) return;
                _sort = value;
                _Refresh_imagePaths();

            }
        }
        private ImagePickerSort _sort;
        public int Count { get => _imagePaths.Count(); }
        public string[] SupportFiles { get; } =
            {
                "JPEG",
                "JPG",
                "PNG"
            };

        public void Reset()
        {
            _index = -1;
        }

        private int _index = -1;
        public Bitmap? Next()
        {
            _index++;
            if (_index > _imagePaths.Length) _index = 0;

            _CheckAllFolder();
            if (_lastFileCount != ImageFolderList.Sum(path => Directory.GetFiles(path).Length) || // tmp file count != now file count, For detection file add and remove
              (_imagePaths.Length > 0 && !File.Exists(_imagePaths[_index]))) // or file not exists, For detection file change
            {
                _Refresh_imagePaths();
                if (_index > _imagePaths.Length) _index = 0;
            }

            if (_imagePaths.Length > 0)
                return new Bitmap(File.OpenRead(_imagePaths.ElementAt(_index)));
            else
                return null;
        }

        private int _randomSeed = (int)DateTime.Now.Ticks;
        private string[] _imagePaths;
        private int _lastFileCount = 0;
        private void _Refresh_imagePaths()
        {
            var imagePathsTmp = ImageFolderList.SelectMany(folderPath => Directory.GetFiles(folderPath));
            _lastFileCount = imagePathsTmp.Count();
            imagePathsTmp = imagePathsTmp.Where(path => SupportFiles.Contains(path.ToUpper().Split('.')[^1]));
            switch (Sort)
            {
                case ImagePickerSort.Name:
                    imagePathsTmp = imagePathsTmp.OrderBy(path => path);
                    break;
                case ImagePickerSort.Name_Reverse:
                    imagePathsTmp = imagePathsTmp.OrderByDescending(path => path);
                    break;
                case ImagePickerSort.CreationTime:
                    imagePathsTmp = imagePathsTmp.OrderBy(path => File.GetCreationTime(path));
                    break;
                case ImagePickerSort.CreationTime_Reverse:
                    imagePathsTmp = imagePathsTmp.OrderByDescending(path => File.GetCreationTime(path));
                    break;
                case ImagePickerSort.LastWriteTime:
                    imagePathsTmp = imagePathsTmp.OrderBy(path => File.GetLastWriteTime(path));
                    break;
                case ImagePickerSort.LastWriteTime_Reverse:
                    imagePathsTmp = imagePathsTmp.OrderByDescending(path => File.GetLastWriteTime(path));
                    break;
                case ImagePickerSort.Random:
                    _randomSeed = (int)DateTime.Now.Ticks;
                    Random random = new Random(_randomSeed);
                    imagePathsTmp = imagePathsTmp.OrderBy(path => random.Next(imagePathsTmp.Count()));
                    break;
                case ImagePickerSort.Random_NotRepeating:
                    Random random2 = new Random(_randomSeed);
                    imagePathsTmp = imagePathsTmp.OrderBy(path => random2.Next(imagePathsTmp.Count()));
                    break;
            }
            _imagePaths = imagePathsTmp.ToArray();
        }
        private void _CheckAllFolder()
        {
            ImageFolderList.RemoveAll(folderPath => !Directory.Exists(folderPath));
        }

    }
}
